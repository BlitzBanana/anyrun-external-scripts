glob '~/.config/{Code - OSS,Code}/User/workspaceStorage/*/workspace.json'
    | each { ||
        $in
        | open
        | get folder
        | str replace "file://" ""
    }
    | uniq
    | where { || $in | path exists }
    | wrap path
    | insert icon {|| $"/usr/share/icons/hicolor/scalable/apps/com.visualstudio.code.oss.svg" }
    | insert title {|| $in.path | path basename}
    | insert description {|| $in.path }
    | insert command {|| $"run-external code ($in.path)" }
    | insert last_used {|| ls -D $in.path | get modified | get 0 }
    | sort-by --reverse last_used
    | to json
