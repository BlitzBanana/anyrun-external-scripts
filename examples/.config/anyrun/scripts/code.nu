glob '~/.config/{Code - OSS,Code}/User/workspaceStorage/*/workspace.json'
    | each { ||
        cat $in
        | from json
        | get folder
        | str replace "file://" ""
        | url decode
        | path expand
    }
    | uniq
    | where { || $in | path exists }
    | enumerate
    | each {|| {
        title: ($in.item | path basename),
        description: $in.item,
        icon: "/usr/share/icons/hicolor/scalable/apps/com.visualstudio.code.oss.svg",
        command: $"run-external code ($in.item)",
        last_used: (ls -D $in.item | get modified | get 0)
    }}
    | sort-by last_used
    | reverse
    | to json
