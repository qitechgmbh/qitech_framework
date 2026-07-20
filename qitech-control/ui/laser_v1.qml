Page {
    ControlGrid {
        title: "Toggles"
        columns: 2

        CommandButton {
            resource: "enable"
            arguments: { index: 0 }
        }

        CommandButton {
            resource: "disable"
            arguments: { index: 0 }
        }

        CommandButton {
            resource: "enable"
            arguments: { index: 1 }
        }

        CommandButton {
            resource: "disable"
            arguments: { index: 1 }
        }

        CommandButton {
            resource: "enable"
            arguments: { index: 2 }
        }

        CommandButton {
            resource: "disable"
            arguments: { index: 2 }
        }

        CommandButton {
            resource: "enable"
            arguments: { index: 3 }
        }

        CommandButton {
            resource: "disable"
            arguments: { index: 4 }
        }
    }
}