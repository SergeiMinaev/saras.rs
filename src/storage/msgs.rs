use once_cell::sync::Lazy;
use async_lock::RwLock;
use sl10n::define_l10n;


define_l10n! {
    CommonErr => {
		en: "Unable to save object.",
		ru: "Не удалось сохранить объект."
	},
}

pub static MSGS: Lazy<Msgs> = Lazy::new(|| Msgs::new());

pub fn t(key: Msg) -> String {
	MSGS.msg(key, "ru")
}
