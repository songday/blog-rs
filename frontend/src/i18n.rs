use std::collections::HashMap;

use fluent::{FluentBundle, FluentResource};
// use unic_langid::LanguageIdentifier;
// use yew_router::navigator::NavigatorKind::Hash;

static EN_US_TEXT: &'static str = include_str!("../resource/i18n/en-US.txt");
static ZH_CN_TEXT: &'static str = include_str!("../resource/i18n/zh-CN.txt");

pub fn get<'a, 'b>(
    accept_language: &'a str,
    message_ids: Vec<&'static str>,
) -> Result<HashMap<&'static str, String>, ()> {
    let (locale, resource) = match accept_language {
        "en-US" => (accept_language, EN_US_TEXT),
        _ => ("zh-CN", ZH_CN_TEXT),
    };
    let ftl_string = resource.to_owned();
    let res = FluentResource::try_new(ftl_string).expect("Failed to parse an FTL string.");

    let lang_id = locale.parse().expect("Parsing failed.");
    let mut bundle = FluentBundle::new(vec![lang_id]);
    bundle
        .add_resource(&res)
        .expect("Failed to add FTL resources to the bundle.");

    let mut result = HashMap::with_capacity(message_ids.len());
    let mut errors = vec![];
    for message_id in message_ids {
        let msg = bundle.get_message(message_id).expect("Message doesn't exist.");
        let pattern = msg.value().expect("Message has no value.");
        let value = bundle.format_pattern(&pattern, None, &mut errors);
        errors.clear();
        result.insert(message_id, value.to_string());
    }
    Ok(result)
}
