#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CommandGroup {
    Navigation,
    Session,
    Integration,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CommandSpec {
    pub id: &'static str,
    pub title: &'static str,
    pub group: CommandGroup,
}

pub const COMMANDS: &[CommandSpec] = &[
    CommandSpec {
        id: "/config",
        title: "Settings Dashboard",
        group: CommandGroup::Navigation,
    },
    CommandSpec {
        id: "/setting",
        title: "Setup Wizard",
        group: CommandGroup::Navigation,
    },
    CommandSpec {
        id: "/provider",
        title: "Provider Management",
        group: CommandGroup::Integration,
    },
    CommandSpec {
        id: "/model",
        title: "Switch Model",
        group: CommandGroup::Integration,
    },
    CommandSpec {
        id: "/status",
        title: "Session Status",
        group: CommandGroup::Session,
    },
    CommandSpec {
        id: "/mode",
        title: "Toggle PLAN/RUN",
        group: CommandGroup::Session,
    },
    CommandSpec {
        id: "/clear",
        title: "Clear Visible Chat",
        group: CommandGroup::Session,
    },
    CommandSpec {
        id: "/compact",
        title: "Compact Context",
        group: CommandGroup::Session,
    },
    CommandSpec {
        id: "/tokens",
        title: "Token Usage",
        group: CommandGroup::Session,
    },
    CommandSpec {
        id: "/help",
        title: "Show Help",
        group: CommandGroup::Navigation,
    },
    CommandSpec {
        id: "/theme",
        title: "Toggle Theme",
        group: CommandGroup::Navigation,
    },
    CommandSpec {
        id: "/workspace",
        title: "Workspace Trust",
        group: CommandGroup::Navigation,
    },
    CommandSpec {
        id: "/mcp",
        title: "MCP Server Management",
        group: CommandGroup::Integration,
    },
    CommandSpec {
        id: "/undo",
        title: "Undo Last smlcli Commit",
        group: CommandGroup::Integration,
    },
    CommandSpec {
        id: "/new",
        title: "New Session",
        group: CommandGroup::Session,
    },
    CommandSpec {
        id: "/resume",
        title: "Resume Session",
        group: CommandGroup::Session,
    },
    CommandSpec {
        id: "/session",
        title: "Session List",
        group: CommandGroup::Session,
    },
    CommandSpec {
        id: "/quit",
        title: "Exit",
        group: CommandGroup::Navigation,
    },
];

pub fn is_known_command(id: &str) -> bool {
    COMMANDS.iter().any(|command| command.id == id)
}

pub fn command_pairs() -> Vec<(&'static str, &'static str)> {
    COMMANDS
        .iter()
        .map(|command| (command.id, command.title))
        .collect()
}
