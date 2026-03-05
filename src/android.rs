use crate::{Browser, BrowserOptions, Error, ErrorKind, Result, TargetType};
use jni::{objects::JString, refs::Global};
use std::process::{Command, Stdio};

/// Deal with opening of browsers on Android. Only [Browser::Default] is supported, and
/// in options, only [BrowserOptions::dry_run] is honoured.
pub(super) fn open_browser_internal(
    browser: Browser,
    target: &TargetType,
    options: &BrowserOptions,
) -> Result<()> {
    // ensure we're opening only http/https urls, failing otherwise
    let url = target.get_http_url()?;

    match browser {
        Browser::Default => open_browser_default(url, options),
        _ => Err(Error::new(
            ErrorKind::NotFound,
            "only default browser supported",
        )),
    }
}

jni::bind_java_type! {
    AUri => "android.net.Uri",
    methods {
        static fn parse(uri: JString) -> AUri
    }

}

jni::bind_java_type! {
    AIntent => "android.content.Intent",
    type_map {
        AUri => "android.net.Uri"
    },
    constructors {
        fn new_with_url(action: JString, uri: AUri)
    },
    fields {
        #[allow(non_snake_case)]
        static ACTION_VIEW: JString,
        #[allow(non_snake_case)]
        static FLAG_ACTIVITY_NEW_TASK: jint
    },
    methods {
        fn add_flags(flags: jint) -> AIntent
    }
}

jni::bind_java_type! {
    AContext => "android.content.Context",
    type_map {
        AIntent => "android.content.Intent"
    },
    methods {
        fn start_activity(intent: AIntent)
    }
}

jni::bind_java_type! {
    AActivity => "android.app.Activity",
    type_map {
        AContext => "android.content.Context"
    },
    is_instance_of {
        activity: AContext
    },
}

#[derive(Debug)]
enum OpenUrlJniError {
    Jni(jni::errors::Error),
    Other(crate::Error),
}
impl From<jni::errors::Error> for OpenUrlJniError {
    fn from(err: jni::errors::Error) -> Self {
        OpenUrlJniError::Jni(err)
    }
}
impl From<crate::Error> for OpenUrlJniError {
    fn from(err: crate::Error) -> Self {
        OpenUrlJniError::Other(err)
    }
}
impl std::fmt::Display for OpenUrlJniError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            OpenUrlJniError::Jni(err) => write!(f, "JNI error: {}", err),
            OpenUrlJniError::Other(err) => write!(f, "{}", err),
        }
    }
}

/// Open the default browser
fn open_browser_default(url: &str, options: &BrowserOptions) -> Result<()> {
    // always return true for a dry run
    if options.dry_run {
        return Ok(());
    }

    // first we try to see if we're in a termux env, because if we are, then
    // the android context may not have been initialized, and it'll panic
    if try_for_termux(url, options).is_ok() {
        return Ok(());
    }

    // Get a `JavaVM` reference for executing Java calls
    let ctx = ndk_context::android_context();
    if ctx.vm().is_null() || ctx.context().is_null() {
        return Err(Error::new(
            ErrorKind::NotFound,
            "Expected to find JVM and context.context.Context reference via ndk_context crate",
        ));
    }
    // Safety: we have checked we have a non-null pointer
    let vm = unsafe { jni::JavaVM::from_raw(ctx.vm() as _) };

    // Note: attach_current_thread will also make sure to catch/clear any Java exceptions that could be
    // thrown while attempting to interact with the Android SDK
    vm.attach_current_thread(|env| -> std::result::Result<(), OpenUrlJniError> {
        // Safety:
        //
        // The docs for `ndk-context` promise that `.context()` will return a reference to an
        // `android.content.Context`
        //
        // The reference associated with ndk-context must implicitly be a global JNI reference
        //
        // Note: although we already check for a `null` `Context` reference above, that's not
        // strictly required for _safety_ here (`null` JNI references are safe).
        //
        // Wrapping the raw reference with a `Cast` here ensures that we can't accidentally delete
        // a global reference that we don't own.
        let context: jni::sys::jobject = ctx.context() as _;
        let context = unsafe { env.as_cast_raw::<Global<AContext>>(&context) }
            .map_err(|e| Error::other(format!("Failed to cast context: {}", e)))?;
        let context: &AContext = &context;

        let action_view = AIntent::ACTION_VIEW(env).map_err(|e| {
            Error::other(format!(
                "Failed to lookup ACTION_VIEW field constant: {}",
                e
            ))
        })?;

        let url = JString::new(env, url)
            .map_err(|e| Error::other(format!("Failed to create JString for URL: {}", e)))?;
        let uri = AUri::parse(env, url)
            .map_err(|e| Error::other(format!("Failed to parse URL: {}", e)))?;

        // Create new ACTION_VIEW intent with the uri
        let intent = AIntent::new_with_url(env, &action_view, &uri)
            .map_err(|e| Error::other(format!("Failed to create ACTION_VIEW intent: {}", e)))?;

        // `ndk-context only` promises to offer an `android.content.Context` reference and so we
        // can't assume that this is an `Activity` (it's also likely to be an `Application`
        // reference)
        //
        // If we have an `Activity` then `startActivity` will automatically try and associate the
        // new activity with the `Task` that holds the existing `Activity`.
        //
        // If we have an `Application` reference then `startActivity` will need the
        // `FLAG_ACTIVITY_NEW_TASK` flag to indicate that we expect the new activity to be started
        // in a new task (and if we don't the system will throw an exception).
        //
        // Note: In practice the end result is likely to be the same either way because a http/https
        // URL will almost certainly open in a separate browser application - not in any `Task`
        // running in the current application.
        let maybe_activity = env.as_cast::<AActivity>(&context);
        match maybe_activity {
            Ok(_activity) => {
                // We have an Activity and so the associated Task is implied
                context.start_activity(env, &intent)?;
            }
            Err(_) => {
                // If we don't have an `Activity` then we need to add the `ACTIVITY_NEW_TASK` flag
                // to the `Intent`, to ensure it can be started from a non-activity context.
                let flag_activity_new_task = AIntent::FLAG_ACTIVITY_NEW_TASK(env)?;
                let intent = intent.add_flags(env, flag_activity_new_task)?;
                context.start_activity(env, &intent)?;
            }
        }

        Ok(())
    })
    .map_err(|e| Error::other(format!("Failed to open URL via Android Intent: {}", e)))?;

    Ok(())
}

/// Attemps to open a browser assuming a termux environment
///
/// See [issue #53](https://github.com/amodm/webbrowser-rs/issues/53)
fn try_for_termux(url: &str, options: &BrowserOptions) -> Result<()> {
    use std::env;
    if env::var("TERMUX_VERSION").is_ok() {
        // return true on dry-run given that termux-open command is guaranteed to be present
        if options.dry_run {
            return Ok(());
        }
        let mut cmd = Command::new("termux-open");
        cmd.arg(url);
        if options.suppress_output {
            cmd.stdin(Stdio::null())
                .stdout(Stdio::null())
                .stderr(Stdio::null());
        }
        cmd.status().and_then(|status| {
            if status.success() {
                Ok(())
            } else {
                Err(Error::other("command present but exited unsuccessfully"))
            }
        })
    } else {
        Err(Error::other("Not a termux environment"))
    }
}
