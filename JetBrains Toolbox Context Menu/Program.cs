using System.Runtime.Versioning;
using System.Text.Json;
using Microsoft.Win32;
using Newtonsoft.Json.Linq;

namespace JetBrains_Toolbox_Context_Menu;

/// <summary>
/// Represents a tool in the JetBrains Toolbox context menu.
/// </summary>
public record Tool(string ChannelId, string ToolId, string ProductCode, string Tag, string DisplayName, string DisplayVersion, string BuildNumber, string InstallLocation, string LaunchCommand);

[SupportedOSPlatform("Windows")]
internal static class Program
{
    /// <summary>
    /// Retrieves the path to the JetBrains Toolbox installation.
    /// </summary>
    /// <returns>
    /// The path to the JetBrains Toolbox installation if found, otherwise null.
    /// </returns>
    private static string? GetJetBrainsToolboxPath()
    {
        RegistryKey? registryKey = Registry.CurrentUser.OpenSubKey(@"SOFTWARE\JetBrains\Toolbox");
        return registryKey?.GetValue(null)?.ToString();
    }

    /// <summary>
    /// Retrieves the list of tools from the state file.
    /// </summary>
    /// <param name="stateFilePath">The path to the state file.</param>
    /// <returns>An array of Tool objects representing the tools.</returns>
    private static Tool[] GetTools(string stateFilePath)
    {
        string json = File.ReadAllText(stateFilePath);
        object? state = JsonSerializer.Deserialize<object>(json);
        if (state is null) return [];
        return JObject.Parse(json)["tools"]?.ToObject<Tool[]>() ?? [];
    }

    /// <summary>
    /// Creates a context menu item for JetBrains Toolbox in the Windows Explorer.
    /// </summary>
    /// <param name="toolboxPath">The path to the JetBrains Toolbox executable file.</param>
    /// <param name="tools">An array of Tool objects representing the available tools in JetBrains Toolbox.</param>
    private static void CreateContextMenu(string toolboxPath, Tool[] tools)
    {
        Console.ForegroundColor = ConsoleColor.Green;
        Console.WriteLine("Creating context menu item for JetBrains Toolbox");
        using (var registryKey = Registry.CurrentUser.CreateSubKey(@"Software\Classes\Directory\Background\shell\JetBrainsToolbox\"))
        {
            Console.ForegroundColor = ConsoleColor.Blue;
            Console.WriteLine("Background Menu");
            registryKey.SetValue("MUIVerb", "Open with JetBrains");
            registryKey.SetValue("SubCommands", "");
            registryKey.SetValue("Icon", $"\"{toolboxPath}\"");
            CreateContextMenuItem(registryKey, tools);
        }

        using (var registryKey = Registry.CurrentUser.CreateSubKey(@"Software\Classes\Directory\shell\JetBrainsToolbox\"))
        {
            Console.ForegroundColor = ConsoleColor.Blue;
            Console.WriteLine("Directory Menu");
            registryKey.SetValue("MUIVerb", "Open with JetBrains");
            registryKey.SetValue("SubCommands", "");
            registryKey.SetValue("Icon", $"\"{toolboxPath}\"");
            CreateContextMenuItem(registryKey, tools);
        }
    }

    /// <summary>
    /// Creates a context menu item for JetBrains Toolbox.
    /// </summary>
    /// <param name="key">The registry key where the context menu item will be created.</param>
    /// <param name="tools">The tools to be added to the context menu.</param>
    private static void CreateContextMenuItem(RegistryKey key, IEnumerable<Tool> tools)
    {
        using var registryKey = key.CreateSubKey("shell");
        Console.ForegroundColor = ConsoleColor.Yellow;
        foreach (Tool tool in tools)
        {
            string exePath = Path.Combine(tool.InstallLocation, tool.LaunchCommand);
            using (var subKey = registryKey.CreateSubKey(tool.DisplayName))
            {
                subKey.SetValue("MUIVerb", tool.DisplayName);
                subKey.SetValue("Icon", $"\"{exePath}\"");
            }

            using (var subKey = registryKey.CreateSubKey($"{tool.DisplayName}\\command"))
            {
                subKey.SetValue("", $"\"{exePath}\" \"%V\"");
                Console.WriteLine($"\t- {tool.DisplayName}");
            }
        }
    }

    private static void RemoveExisingContextMenu()
    {
        Registry.CurrentUser.DeleteSubKeyTree(@"Software\Classes\Directory\Background\shell\JetBrainsToolbox", false);
        Registry.CurrentUser.DeleteSubKeyTree(@"Software\Classes\Directory\shell\JetBrainsToolbox", false);
    }

    private static void Main()
    {
        string? jetBrainsToolboxPath = GetJetBrainsToolboxPath();
        if (jetBrainsToolboxPath is null)
        {
            Console.WriteLine("JetBrains Toolbox not found");
            return;
        }

        string jetBrainsToolboxExecutablePath = Path.Combine(jetBrainsToolboxPath, "jetbrains-toolbox.exe");

        if (!File.Exists(jetBrainsToolboxExecutablePath))
        {
            Console.ForegroundColor = ConsoleColor.Red;
            Console.Error.WriteLine("jetbrains-toolbox.exe not found");
            Console.ResetColor();

            Environment.Exit(1);
            return;
        }

        string stateFilePath = Path.Combine(Directory.GetParent(jetBrainsToolboxPath)?.FullName ?? jetBrainsToolboxPath, "state.json");
        if (!File.Exists(stateFilePath))
        {
            Console.ForegroundColor = ConsoleColor.Red;
            Console.Error.WriteLine("state.json not found");
            Console.ResetColor();

            Environment.Exit(1);
            return;
        }

        Console.WriteLine("Removing existing context menu items, if any...");
        RemoveExisingContextMenu();

        Tool[] tools = GetTools(stateFilePath);

        CreateContextMenu(jetBrainsToolboxExecutablePath, tools);

        Console.WriteLine("Done!");

        // pause at the end
        Console.ResetColor();
        Console.Write("Press any key to continue . . .");
        Console.ReadKey();
    }
}