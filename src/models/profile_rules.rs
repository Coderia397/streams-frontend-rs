//! The rules for a set of profiles, in one place. The account database holds the same ones (a trigger refuses the main profile being deleted or being
//! a Kids profile, and an account without a profile), so screens that follow them never show a change the account would undo.
use crate::models::profile::Profile;

/// Rules every screen and the account share (the database holds the same ones): one profile is the main profile, it is never a Kids profile
/// and it cannot be deleted, and an account always keeps at least one profile.
pub fn normalize(list: &mut [Profile]) {
    if list.is_empty() {
        return;
    }
    let main = list.iter().position(|p| p.is_default == Some(true) && !p.is_kids).or_else(|| list.iter().position(|p| !p.is_kids)).unwrap_or(0);
    for (i, p) in list.iter_mut().enumerate() {
        p.is_default = Some(i == main);
    }
    list[main].is_kids = false;
}

fn is_main(list: &[Profile], id: &str) -> bool {
    list.iter().any(|p| p.id == id && p.is_default == Some(true))
}

/// The main profile and the last profile cannot be deleted.
pub fn can_delete(list: &[Profile], id: &str) -> bool {
    list.len() > 1 && !is_main(list, id)
}

/// The main profile cannot be switched to Kids.
pub fn can_be_kids(list: &[Profile], id: &str) -> bool {
    !is_main(list, id)
}

#[cfg(test)]
mod rules {
    use super::*;
    use crate::models::profile::Profile;

    fn p(id: &str, kids: bool, main: Option<bool>) -> Profile {
        Profile { id: id.into(), name: id.into(), avatar_url: None, is_kids: kids, is_default: main, pin: None, sort_order: 0 }
    }

    #[test]
    fn normalize_picks_one_main_and_it_is_not_kids() {
        let mut list = vec![p("a", true, None), p("b", false, None), p("c", false, None)];
        normalize(&mut list);
        assert_eq!(list.iter().filter(|x| x.is_default == Some(true)).count(), 1);
        assert_eq!(list[1].is_default, Some(true));
        let mut only_kids = vec![p("a", true, Some(true))];
        normalize(&mut only_kids);
        assert!(!only_kids[0].is_kids);
    }

    #[test]
    fn main_and_last_cannot_be_deleted_main_cannot_be_kids() {
        let list = vec![p("a", false, Some(true)), p("b", false, Some(false))];
        assert!(!can_delete(&list, "a") && can_delete(&list, "b"));
        assert!(!can_be_kids(&list, "a") && can_be_kids(&list, "b"));
        assert!(!can_delete(&list[..1], "a"));
    }
}
