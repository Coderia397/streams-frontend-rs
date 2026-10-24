//! The addresses each share target opens, as each service documents them (checked 2026-10-24, note 116): WhatsApp's wa.me text link,
//! the sms: body link, X's post composer (docs.x.com, web intent) and Snapchat's share link (developers.snap.com, Share Link to
//! Snapchat): on a phone the app opens its camera with the link attached as a card, on a computer Snapchat's web chat sends it to
//! friends. The card shows the page's name and site, read from the page, so a link the app cannot reach (localhost) opens a bare scan
//! pop-up instead. Instagram has no such address; it is reached through the system share sheet (sheet.rs).

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
    format!("https://www.snapchat.com/share?link={}", encode(url))
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
        assert_eq!(snapchat(url), "https://www.snapchat.com/share?link=https%3A%2F%2Fstreams.watch%2Ftitle%2Fmovie-194");
        assert_eq!(whatsapp("Up", url), "https://wa.me/?text=Up%20https%3A%2F%2Fstreams.watch%2Ftitle%2Fmovie-194");
        assert_eq!(messages("Up", url), "sms:?&body=Up%20https%3A%2F%2Fstreams.watch%2Ftitle%2Fmovie-194");
    }
}
