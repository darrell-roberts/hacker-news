//! Macos system access.
use crate::Theme;
use anyhow::Context;
use log::info;
use objc2::rc::Retained;
use objc2_foundation::{ns_string, NSString, NSUserDefaults};

pub fn initial_theme() -> anyhow::Result<Theme> {
    unsafe {
        let style = NSUserDefaults::standardUserDefaults()
            .persistentDomainForName(ns_string!("Apple Global Domain"))
            .context("Failed to lookup global domain")?
            .objectForKey(ns_string!("AppleInterfaceStyle"));

        let Some(style) = style else {
            info!("No style found. Using light theme.");
            return Ok(Theme::Light);
        };

        let style = Retained::cast_unchecked::<NSString>(style);
        info!("Macos interface style: {style}");
        let dark_mode = style.isEqualToString(ns_string!("Dark"));

        Ok(if dark_mode { Theme::Dark } else { Theme::Light })
    }
}

// pub fn subscribe_to_theme_changes(callback: impl Fn(Theme) + 'static) {
//     unsafe {
//         let center: *mut objc2::runtime::AnyObject =
//             msg_send![class!(NSDistributedNotificationCenter), defaultCenter];
//         let notification_name = ns_string!("AppleInterfaceThemeChangedNotification");
//         let callback = std::sync::Arc::new(callback);

//         extern "C" fn theme_changed_callback(
//             _self: &objc2::runtime::AnyObject,
//             _cmd: Sel,
//             _notification: *mut objc2::runtime::AnyObject,
//         ) {
//             // Query the current theme and invoke the callback
//             if let Ok(theme) = initial_theme() {
//                 // SAFETY: The callback is stored in a thread-safe Arc and can be safely cloned.
//                 let callback = unsafe {
//                     let ptr = _self as *const objc2::runtime::AnyObject
//                         as *const std::sync::Arc<dyn Fn(Theme)>;
//                     (*ptr).clone()
//                 };
//                 callback(theme);
//             }
//         }

//         let superclass = class!(NSObject);
//         let mut builder =
//             objc2::runtime::ClassBuilder::new("ThemeChangeObserver", superclass).unwrap();

//         builder.add_method(
//             sel!(themeChanged:),
//             theme_changed_callback
//                 as extern "C" fn(&objc2::runtime::AnyObject, Sel, *mut objc2::runtime::AnyObject),
//         );

//         let observer_class = builder.register();

//         let observer: *mut objc2::runtime::AnyObject = msg_send![observer_class, new];
//         let observer_ptr = observer as *mut std::sync::Arc<dyn Fn(Theme)>;
//         std::ptr::write(observer_ptr, callback.clone());

//         let _: () = msg_send![center, addObserver: observer
//                                              selector: sel!(themeChanged:)
//                                                  name: notification_name
//                                                object: std::ptr::null::<objc2::runtime::AnyObject>()];
//     }
// }
