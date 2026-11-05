use super::register_yo_scheme;
use anyhow::Result;
use gpui::{AppContext as _, AsyncApp, Context, PromptButton, PromptLevel, Window, actions};
use i18n::t;
use release_channel::ReleaseChannel;
use std::ops::Deref;
use std::path::{Path, PathBuf};
use util::ResultExt;
use workspace::notifications::simple_message_notification::MessageNotification;
use workspace::notifications::{DetachAndPromptErr, NotificationId};
use workspace::{Toast, Workspace};

actions!(
    cli,
    [
        /// Installs the Yo CLI tool to the system PATH.
        InstallCliBinary,
    ]
);

const CANT_INSTALL_DOCS_URL: &str =
    "https://github.com/magalab/yo/blob/main/docs/INSTALL.md#command-line";

/// Attempts to install the CLI symlink. Returns the installed path on success,
/// or `None` if the user dismissed the macOS administrator authentication
/// prompt. Returns an error if the install could not be completed, most
/// commonly because the user is not an admin.
async fn install_script(cx: &AsyncApp) -> Result<Option<PathBuf>> {
    let cli_path = cx.update(|cx| cx.path_for_auxiliary_executable("cli"))?;
    let yo_link_path = Path::new("/usr/local/bin/yo");
    let Some(bin_dir_path) = yo_link_path.parent() else {
        anyhow::bail!("the Yo CLI link has no parent directory");
    };
    match smol::fs::symlink_metadata(yo_link_path).await {
        Ok(metadata) if metadata.file_type().is_symlink() => {
            if smol::fs::read_link(yo_link_path).await? == cli_path {
                return Ok(Some(yo_link_path.into()));
            }
        }
        Ok(_) => anyhow::bail!(
            "refusing to replace existing path {}",
            yo_link_path.display()
        ),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
        Err(error) => return Err(error.into()),
    }

    let install_result = async {
        smol::fs::create_dir_all(bin_dir_path).await?;
        match smol::fs::remove_file(yo_link_path).await {
            Ok(()) => {}
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
            Err(error) => return Err(error),
        }
        smol::fs::unix::symlink(&cli_path, yo_link_path).await
    }
    .await;
    match install_result {
        Ok(()) => return Ok(Some(yo_link_path.into())),
        Err(error) if error.kind() == std::io::ErrorKind::PermissionDenied => {}
        Err(error) => return Err(error.into()),
    }

    let shell_quote = |path: &Path| format!("'{}'", path.to_string_lossy().replace("'", "'\"'\"'"));
    let command = format!(
        "mkdir -p {} && ln -sfn {} {}",
        shell_quote(bin_dir_path),
        shell_quote(&cli_path),
        shell_quote(yo_link_path),
    );
    let script = format!(
        "do shell script \"{}\" with administrator privileges",
        command.replace('\\', "\\\\").replace('\"', "\\\""),
    );
    let output = smol::process::Command::new("/usr/bin/osascript")
        .args(["-e", &script])
        .output()
        .await?;

    if output.status.success() {
        return Ok(Some(yo_link_path.into()));
    }

    let stderr = String::from_utf8_lossy(&output.stderr);
    if stderr.contains("User canceled") || stderr.contains("-128") {
        return Ok(None);
    }

    anyhow::bail!("error running osascript: {}", stderr.trim());
}

pub fn install_cli_binary(window: &mut Window, cx: &mut Context<Workspace>) {
    cx.spawn_in(window, async move |workspace, cx| {
        if cfg!(any(target_os = "linux", target_os = "freebsd")) {
            let prompt = cx.prompt(
                PromptLevel::Warning,
                &t!("CLI should already be installed"),
                Some(&t!(
                    "If you installed Yo from our official release add ~/.local/bin to your PATH.\n\nIf you installed Yo from a different source like your package manager, then you may need to create an alias/symlink manually.\n\nDepending on your package manager, the CLI might be named zeditor, zedit, zed-editor or something else."
                )),
                &[PromptButton::ok(t!("OK"))],
            );
            cx.background_spawn(prompt).detach();
            return Ok(());
        }
        let path = match install_script(cx.deref()).await {
            Ok(Some(path)) => path,
            // The user dismissed the administrator prompt; nothing to do.
            Ok(None) => return Ok(()),
            Err(error) => {
                log::error!("failed to install Yo CLI: {error:#}");
                workspace.update(cx, |workspace, cx| {
                    struct CliInstallFailed;

                    workspace.show_notification(
                        NotificationId::unique::<CliInstallFailed>(),
                        cx,
                        |cx| {
                            cx.new(|cx| {
                                MessageNotification::new(
                                    t!("You can add `yo` to your PATH manually."),
                                    cx,
                                )
                                .with_title(t!("Couldn't install the Yo CLI"))
                                .more_info_message(t!("Show me how"))
                                .more_info_url(CANT_INSTALL_DOCS_URL)
                            })
                        },
                    );
                })?;
                return Ok(());
            }
        };

        workspace.update_in(cx, |workspace, _, cx| {
            struct InstalledZedCli;

            workspace.show_toast(
                Toast::new(
                    NotificationId::unique::<InstalledZedCli>(),
                    t!(
                        "Installed `yo` to {path}. You can launch {app} from your terminal.",
                        path = path.to_string_lossy(),
                        app = ReleaseChannel::global(cx).display_name()
                    )
                    .to_string(),
                ),
                cx,
            )
        })?;
        register_yo_scheme(cx).await.log_err();
        Ok(())
    })
    .detach_and_prompt_err(&t!("Cannot install the Yo CLI"), window, cx, |_, _, _| None);
}
