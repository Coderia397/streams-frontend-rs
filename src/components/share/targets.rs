//! The addresses each share target opens, as each service documents or uses them (checked 2026-10-24, note 116): WhatsApp's wa.me
//! text link, the sms: body link, X's post composer (docs.x.com, web intent), and Snapchat's scan page with the link attached, which
//! Snap's own share button opens (its sdk.snapkit.com create.js: the page itself on a phone, which hands it to the app, a window with a
//! Snapcode on a computer). Instagram has no such address; it is reached through the system share sheet (sheet.rs).

/// `s` encoded as JavaScript's encodeURIComponent does (letters, digits and `-_.!~*'()` kept).
pub fn encode(s: &str) -> String {
    let mut out = String::with_capacity(s.len() * 3);
    for b in s.bytes() {
        if b.is_ascii_alphanumeric() || b"-_.!~*'()".contains(&b) {
            out.push(b as char);
        } else {
            out.push_str(&format!("%{b:02X}"));
        }
    }
    out
}

pub fn whatsapp(title: &str, url: &str) -> String {
    format!("https://wa.me/?text={}", encode(&format!("{title} {url}")))
}

/// `?&body=` is read by both Android's and Apple's messages apps.
pub fn messages(title: &str, url: &str) -> String {
    format!("sms:?&body={}", encode(&format!("{title} {url}")))
}

pub fn x(title: &str, url: &str) -> String {
    format!("https://x.com/intent/tweet?text={}&url={}", encode(title), encode(url))
}

pub fn snapchat(url: &str) -> String {
    format!("https://snapchat.com/scan?attachmentUrl={}", encode(url))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn encodes_as_encode_uri_component() {
        assert_eq!(encode("Tom & Jerry: 100% (new)!"), "Tom%20%26%20Jerry%3A%20100%25%20(new)!");
        assert_eq!(encode("https://streams.watch/title/tv-1399?x=1"), "https%3A%2F%2Fstreams.watch%2Ftitle%2Ftv-1399%3Fx%3D1");
        assert_eq!(encode("Amélie"), "Am%C3%A9lie");
    }

    #[test]
    fn target_addresses() {
        let url = "https://streams.watch/title/movie-194";
        assert_eq!(x("Amélie", url), "https://x.com/intent/tweet?text=Am%C3%A9lie&url=https%3A%2F%2Fstreams.watch%2Ftitle%2Fmovie-194");
        assert_eq!(snapchat(url), "https://snapchat.com/scan?attachmentUrl=https%3A%2F%2Fstreams.watch%2Ftitle%2Fmovie-194");
        assert_eq!(whatsapp("Up", url), "https://wa.me/?text=Up%20https%3A%2F%2Fstreams.watch%2Ftitle%2Fmovie-194");
        assert_eq!(messages("Up", url), "sms:?&body=Up%20https%3A%2F%2Fstreams.watch%2Ftitle%2Fmovie-194");
    }
}
