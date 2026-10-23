//! Static avatar definitions and categories ported from `streams-frontend/data/avatars.ts`.
//! Zero-cost compile-time static slices for WebAssembly runtime efficiency.

use serde::Serialize;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct Avatar {
    pub name: &'static str,
    pub url: &'static str,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct AvatarCategory {
    pub id: &'static str,
    pub name: &'static str,
    /// True for a set that a Kids profile may choose from.
    pub kids: bool,
    pub avatars: &'static [Avatar],
}

pub static AVATAR_CATEGORIES: &[AvatarCategory] = &[
    AvatarCategory {
        id: "bridgerton",
        name: "Bridgerton",
        kids: false,
        avatars: &[
            Avatar { name: "Anthony Bridgerton", url: "https://emvyjhleyhimxxdqybsn.supabase.co/storage/v1/object/public/avatars/1KzRtMnyHwlJYwjr09S3UjhLtEg51W-Lr.webp" },
            Avatar { name: "Benedict Bridgerton", url: "https://emvyjhleyhimxxdqybsn.supabase.co/storage/v1/object/public/avatars/1YlWDXxAhiKlc6r641vEVqTqFsXobC__-.webp" },
            Avatar { name: "Colin Bridgerton", url: "https://emvyjhleyhimxxdqybsn.supabase.co/storage/v1/object/public/avatars/1ay24h5pSUfQ339nf1yvYtuMVLsY-L_r5.webp" },
            Avatar { name: "Daphne Basset", url: "https://emvyjhleyhimxxdqybsn.supabase.co/storage/v1/object/public/avatars/1iloFbK5eBE0JwAYsyEyZEYMahIaWLiZL.webp" },
            Avatar { name: "Eloise Bridgerton", url: "https://emvyjhleyhimxxdqybsn.supabase.co/storage/v1/object/public/avatars/1eNNCBDTROwaoishIx9_fkx8YFucs12NE.webp" },
            Avatar { name: "Francesca Bridgerton", url: "https://emvyjhleyhimxxdqybsn.supabase.co/storage/v1/object/public/avatars/1Ocb11OmSHJlRfzT6w97e_yuCp0B1X3EC.webp" },
            Avatar { name: "Kathani Bridgerton", url: "https://emvyjhleyhimxxdqybsn.supabase.co/storage/v1/object/public/avatars/1QgeRATcBSnH8prA5S3Hqnxm3VwB7YB9G.webp" },
            Avatar { name: "Lady Danbury", url: "https://emvyjhleyhimxxdqybsn.supabase.co/storage/v1/object/public/avatars/1qOo2XsRjVZsZQWUdLUJpHwuGEBRYlPQX.webp" },
            Avatar { name: "Penelope Featherington", url: "https://emvyjhleyhimxxdqybsn.supabase.co/storage/v1/object/public/avatars/1Z__4y8VugluuVFwRdqG6eKaxwuy5rzc6.webp" },
            Avatar { name: "Queen Charlotte", url: "https://emvyjhleyhimxxdqybsn.supabase.co/storage/v1/object/public/avatars/1Bvp9-Q7K6KqKxxxOz4l75ksVsJ4NhYmR.webp" },
            Avatar { name: "Simon Basset", url: "https://emvyjhleyhimxxdqybsn.supabase.co/storage/v1/object/public/avatars/1bMUWIh4lqfpSrf5ifpdZ29GKC-gFCtJH.webp" },
            Avatar { name: "Sophie Baek", url: "https://emvyjhleyhimxxdqybsn.supabase.co/storage/v1/object/public/avatars/1PVt6TyXP55YMM9c5tJNXev9Lsw1mDBlF.webp" },
            Avatar { name: "Violet Bridgerton", url: "https://emvyjhleyhimxxdqybsn.supabase.co/storage/v1/object/public/avatars/1CIWh6dWbYek4Z7tLQVXkHEzBdN54xVFu.webp" },
        ],
    },
    AvatarCategory {
        id: "one-piece",
        name: "One Piece",
        kids: false,
        avatars: &[
            Avatar { name: "Monkey D. Luffy", url: "https://emvyjhleyhimxxdqybsn.supabase.co/storage/v1/object/public/avatars/1CCWWd9W3ODzxAn1lJ6TsKRYyAxdLxeq8.webp" },
            Avatar { name: "Zoro", url: "https://emvyjhleyhimxxdqybsn.supabase.co/storage/v1/object/public/avatars/1HqM0dZKFN99eJw015CWCeZxpqix3EgT0.webp" },
            Avatar { name: "Nami", url: "https://emvyjhleyhimxxdqybsn.supabase.co/storage/v1/object/public/avatars/1zqX1Q-0BIrG0taII5kqsL_zRNt1oVYE6.webp" },
            Avatar { name: "Sanji", url: "https://emvyjhleyhimxxdqybsn.supabase.co/storage/v1/object/public/avatars/1ZL2FXFwrOjAiHuOXZ0SUu2gXZy3Jw3ts.webp" },
            Avatar { name: "Chopper", url: "https://emvyjhleyhimxxdqybsn.supabase.co/storage/v1/object/public/avatars/1yZOQdzM_MPyWOpMgjqfd0S73KIoDAjLl.webp" },
            Avatar { name: "Usopp", url: "https://emvyjhleyhimxxdqybsn.supabase.co/storage/v1/object/public/avatars/1T63TmoapNx9DLbTrOUWtu93XpqZkhQM5.webp" },
            Avatar { name: "Arlong", url: "https://emvyjhleyhimxxdqybsn.supabase.co/storage/v1/object/public/avatars/1bUqGSDkhtgw1FtXZJqD5flQ-AMYjrBxa.webp" },
            Avatar { name: "Alvida", url: "https://emvyjhleyhimxxdqybsn.supabase.co/storage/v1/object/public/avatars/1kRyzph--pyxtTGjEU35nt1GDPz70fMrU.webp" },
            Avatar { name: "Shanks", url: "https://emvyjhleyhimxxdqybsn.supabase.co/storage/v1/object/public/avatars/1Sr0hpD0rTFJyyBbCWoKC0x6oD-rke-pU.webp" },
            Avatar { name: "Going Merry", url: "https://emvyjhleyhimxxdqybsn.supabase.co/storage/v1/object/public/avatars/1my31WNqaaUo5v34oK08r_3o30Yvc6GAf.webp" },
            Avatar { name: "Jolly Roger", url: "https://emvyjhleyhimxxdqybsn.supabase.co/storage/v1/object/public/avatars/18n1qSVxKIFQwX308WEsHC78-SlnSTRBu.webp" },
        ],
    },
    AvatarCategory {
        id: "peaky-blinders",
        name: "Peaky Blinders",
        kids: false,
        avatars: &[
            Avatar { name: "Tommy Shelby", url: "https://emvyjhleyhimxxdqybsn.supabase.co/storage/v1/object/public/avatars/1wW1ox6Uc1g368rqZ5CAphVSH84KW711n.webp" },
            Avatar { name: "Duke", url: "https://emvyjhleyhimxxdqybsn.supabase.co/storage/v1/object/public/avatars/1lv3S6zkHP2cj7x1_z1Xas6cwN86bsrb-.webp" },
            Avatar { name: "Hayden Stagg", url: "https://emvyjhleyhimxxdqybsn.supabase.co/storage/v1/object/public/avatars/1abCdQHYZ4A6thk6WvT0LMs7fgNVoXo1b.webp" },
            Avatar { name: "John Beckett", url: "https://emvyjhleyhimxxdqybsn.supabase.co/storage/v1/object/public/avatars/1fMP16GVuLI_YE58zqgzokzrTrhCoN3cW.webp" },
            Avatar { name: "Kaulo Chiriklo", url: "https://emvyjhleyhimxxdqybsn.supabase.co/storage/v1/object/public/avatars/1e1jV3_MfCKrv25cEy-y14HxVD3ChdJtT.webp" },
        ],
    },
    AvatarCategory {
        id: "lucifer",
        name: "Lucifer",
        kids: false,
        avatars: &[
            Avatar { name: "Lucifer Morningstar", url: "https://emvyjhleyhimxxdqybsn.supabase.co/storage/v1/object/public/avatars/1vuBOiPd9DknqbZpUXgMUEAUwZiGee52g.webp" },
            Avatar { name: "Chloe Decker", url: "https://emvyjhleyhimxxdqybsn.supabase.co/storage/v1/object/public/avatars/1KtbqNziC8SxPDUJfMS2NGKhoaKaPUdos.webp" },
            Avatar { name: "Mazikeen", url: "https://emvyjhleyhimxxdqybsn.supabase.co/storage/v1/object/public/avatars/1hygVLyg-7PsCkE1fdRByTje4OmgE7773.webp" },
            Avatar { name: "Amenadiel", url: "https://emvyjhleyhimxxdqybsn.supabase.co/storage/v1/object/public/avatars/1dfWobx2-vWsArr2lQeZ_guWI2ZPvMQNo.webp" },
        ],
    },
    AvatarCategory {
        id: "classics",
        name: "P-Stream Classics",
        kids: true,
        avatars: &[
            Avatar { name: "Blue Fluffball", url: "https://emvyjhleyhimxxdqybsn.supabase.co/storage/v1/object/public/avatars/1i3UrprAcfhKSNaSwFE1FXwTD6NXOfjaV.webp" },
            Avatar { name: "Gray Fluffball", url: "https://emvyjhleyhimxxdqybsn.supabase.co/storage/v1/object/public/avatars/1gKlTO0SLMJzk0RUqihW7n4ovugEZj9Jf.webp" },
            Avatar { name: "Orange Fluffball", url: "https://emvyjhleyhimxxdqybsn.supabase.co/storage/v1/object/public/avatars/1I-MhzW-S8sQkJ72QOKQelNHnas41VWkn.webp" },
            Avatar { name: "Bubblegum Princess", url: "https://emvyjhleyhimxxdqybsn.supabase.co/storage/v1/object/public/avatars/1ltTBpxXy_QxWDIJyyJHSYtwRemSb_fYb.webp" },
            Avatar { name: "Green Alien", url: "https://emvyjhleyhimxxdqybsn.supabase.co/storage/v1/object/public/avatars/1_shb0mnchPaWk-F9anvInjBBRbGXJF7Z.webp" },
            Avatar { name: "Panda Face", url: "https://emvyjhleyhimxxdqybsn.supabase.co/storage/v1/object/public/avatars/1MOcMHPqN0hFbkpoqjdirZl4jgI5VFVqo.webp" },
            Avatar { name: "Red Anger", url: "https://emvyjhleyhimxxdqybsn.supabase.co/storage/v1/object/public/avatars/198aosLkzeCyglhaKy5vPMeWktSJhFui_.webp" },
            Avatar { name: "Yellow Chicken", url: "https://emvyjhleyhimxxdqybsn.supabase.co/storage/v1/object/public/avatars/1ZYyoo8gUHeugXIa5ciA6pJySe3OPdkNB.webp" },
            Avatar { name: "Red Smile", url: "https://emvyjhleyhimxxdqybsn.supabase.co/storage/v1/object/public/avatars/classic-red-smile.webp" },
            Avatar { name: "Dark Blue Smile", url: "https://emvyjhleyhimxxdqybsn.supabase.co/storage/v1/object/public/avatars/classic-dark-grey-smile.webp" },
            Avatar { name: "Yellow Smile", url: "https://emvyjhleyhimxxdqybsn.supabase.co/storage/v1/object/public/avatars/classic-yellow-smile.webp" },
            Avatar { name: "Green Smile", url: "https://emvyjhleyhimxxdqybsn.supabase.co/storage/v1/object/public/avatars/classic-green-smile.webp" },
            Avatar { name: "Purple Smile", url: "https://emvyjhleyhimxxdqybsn.supabase.co/storage/v1/object/public/avatars/classic-purple-smile.webp" },
            Avatar { name: "Pink Smile", url: "https://emvyjhleyhimxxdqybsn.supabase.co/storage/v1/object/public/avatars/classic-pink-smile.webp" },
        ],
    },
];

pub static ALL_AVATARS: &[&str] = &[
    "https://emvyjhleyhimxxdqybsn.supabase.co/storage/v1/object/public/avatars/classic-red-smile.webp",
    "https://emvyjhleyhimxxdqybsn.supabase.co/storage/v1/object/public/avatars/classic-dark-grey-smile.webp",
    "https://emvyjhleyhimxxdqybsn.supabase.co/storage/v1/object/public/avatars/classic-yellow-smile.webp",
    "https://emvyjhleyhimxxdqybsn.supabase.co/storage/v1/object/public/avatars/classic-green-smile.webp",
    "https://emvyjhleyhimxxdqybsn.supabase.co/storage/v1/object/public/avatars/classic-purple-smile.webp",
    "https://emvyjhleyhimxxdqybsn.supabase.co/storage/v1/object/public/avatars/classic-pink-smile.webp",
    "https://emvyjhleyhimxxdqybsn.supabase.co/storage/v1/object/public/avatars/1KzRtMnyHwlJYwjr09S3UjhLtEg51W-Lr.webp",
    "https://emvyjhleyhimxxdqybsn.supabase.co/storage/v1/object/public/avatars/1YlWDXxAhiKlc6r641vEVqTqFsXobC__-.webp",
    "https://emvyjhleyhimxxdqybsn.supabase.co/storage/v1/object/public/avatars/1ay24h5pSUfQ339nf1yvYtuMVLsY-L_r5.webp",
    "https://emvyjhleyhimxxdqybsn.supabase.co/storage/v1/object/public/avatars/1iloFbK5eBE0JwAYsyEyZEYMahIaWLiZL.webp",
    "https://emvyjhleyhimxxdqybsn.supabase.co/storage/v1/object/public/avatars/1eNNCBDTROwaoishIx9_fkx8YFucs12NE.webp",
    "https://emvyjhleyhimxxdqybsn.supabase.co/storage/v1/object/public/avatars/1Ocb11OmSHJlRfzT6w97e_yuCp0B1X3EC.webp",
    "https://emvyjhleyhimxxdqybsn.supabase.co/storage/v1/object/public/avatars/1QgeRATcBSnH8prA5S3Hqnxm3VwB7YB9G.webp",
    "https://emvyjhleyhimxxdqybsn.supabase.co/storage/v1/object/public/avatars/1qOo2XsRjVZsZQWUdLUJpHwuGEBRYlPQX.webp",
    "https://emvyjhleyhimxxdqybsn.supabase.co/storage/v1/object/public/avatars/1Z__4y8VugluuVFwRdqG6eKaxwuy5rzc6.webp",
    "https://emvyjhleyhimxxdqybsn.supabase.co/storage/v1/object/public/avatars/1Bvp9-Q7K6KqKxxxOz4l75ksVsJ4NhYmR.webp",
    "https://emvyjhleyhimxxdqybsn.supabase.co/storage/v1/object/public/avatars/1bMUWIh4lqfpSrf5ifpdZ29GKC-gFCtJH.webp",
    "https://emvyjhleyhimxxdqybsn.supabase.co/storage/v1/object/public/avatars/1PVt6TyXP55YMM9c5tJNXev9Lsw1mDBlF.webp",
    "https://emvyjhleyhimxxdqybsn.supabase.co/storage/v1/object/public/avatars/1CIWh6dWbYek4Z7tLQVXkHEzBdN54xVFu.webp",
    "https://emvyjhleyhimxxdqybsn.supabase.co/storage/v1/object/public/avatars/1CCWWd9W3ODzxAn1lJ6TsKRYyAxdLxeq8.webp",
    "https://emvyjhleyhimxxdqybsn.supabase.co/storage/v1/object/public/avatars/1HqM0dZKFN99eJw015CWCeZxpqix3EgT0.webp",
    "https://emvyjhleyhimxxdqybsn.supabase.co/storage/v1/object/public/avatars/1zqX1Q-0BIrG0taII5kqsL_zRNt1oVYE6.webp",
    "https://emvyjhleyhimxxdqybsn.supabase.co/storage/v1/object/public/avatars/1ZL2FXFwrOjAiHuOXZ0SUu2gXZy3Jw3ts.webp",
    "https://emvyjhleyhimxxdqybsn.supabase.co/storage/v1/object/public/avatars/1yZOQdzM_MPyWOpMgjqfd0S73KIoDAjLl.webp",
    "https://emvyjhleyhimxxdqybsn.supabase.co/storage/v1/object/public/avatars/1T63TmoapNx9DLbTrOUWtu93XpqZkhQM5.webp",
    "https://emvyjhleyhimxxdqybsn.supabase.co/storage/v1/object/public/avatars/1bUqGSDkhtgw1FtXZJqD5flQ-AMYjrBxa.webp",
    "https://emvyjhleyhimxxdqybsn.supabase.co/storage/v1/object/public/avatars/1kRyzph--pyxtTGjEU35nt1GDPz70fMrU.webp",
    "https://emvyjhleyhimxxdqybsn.supabase.co/storage/v1/object/public/avatars/1Sr0hpD0rTFJyyBbCWoKC0x6oD-rke-pU.webp",
    "https://emvyjhleyhimxxdqybsn.supabase.co/storage/v1/object/public/avatars/1my31WNqaaUo5v34oK08r_3o30Yvc6GAf.webp",
    "https://emvyjhleyhimxxdqybsn.supabase.co/storage/v1/object/public/avatars/18n1qSVxKIFQwX308WEsHC78-SlnSTRBu.webp",
    "https://emvyjhleyhimxxdqybsn.supabase.co/storage/v1/object/public/avatars/1wW1ox6Uc1g368rqZ5CAphVSH84KW711n.webp",
    "https://emvyjhleyhimxxdqybsn.supabase.co/storage/v1/object/public/avatars/1lv3S6zkHP2cj7x1_z1Xas6cwN86bsrb-.webp",
    "https://emvyjhleyhimxxdqybsn.supabase.co/storage/v1/object/public/avatars/1abCdQHYZ4A6thk6WvT0LMs7fgNVoXo1b.webp",
    "https://emvyjhleyhimxxdqybsn.supabase.co/storage/v1/object/public/avatars/1fMP16GVuLI_YE58zqgzokzrTrhCoN3cW.webp",
    "https://emvyjhleyhimxxdqybsn.supabase.co/storage/v1/object/public/avatars/1e1jV3_MfCKrv25cEy-y14HxVD3ChdJtT.webp",
    "https://emvyjhleyhimxxdqybsn.supabase.co/storage/v1/object/public/avatars/1vuBOiPd9DknqbZpUXgMUEAUwZiGee52g.webp",
    "https://emvyjhleyhimxxdqybsn.supabase.co/storage/v1/object/public/avatars/1KtbqNziC8SxPDUJfMS2NGKhoaKaPUdos.webp",
    "https://emvyjhleyhimxxdqybsn.supabase.co/storage/v1/object/public/avatars/1hygVLyg-7PsCkE1fdRByTje4OmgE7773.webp",
    "https://emvyjhleyhimxxdqybsn.supabase.co/storage/v1/object/public/avatars/1dfWobx2-vWsArr2lQeZ_guWI2ZPvMQNo.webp",
    "https://emvyjhleyhimxxdqybsn.supabase.co/storage/v1/object/public/avatars/1i3UrprAcfhKSNaSwFE1FXwTD6NXOfjaV.webp",
    "https://emvyjhleyhimxxdqybsn.supabase.co/storage/v1/object/public/avatars/1gKlTO0SLMJzk0RUqihW7n4ovugEZj9Jf.webp",
    "https://emvyjhleyhimxxdqybsn.supabase.co/storage/v1/object/public/avatars/1I-MhzW-S8sQkJ72QOKQelNHnas41VWkn.webp",
    "https://emvyjhleyhimxxdqybsn.supabase.co/storage/v1/object/public/avatars/1ltTBpxXy_QxWDIJyyJHSYtwRemSb_fYb.webp",
    "https://emvyjhleyhimxxdqybsn.supabase.co/storage/v1/object/public/avatars/1_shb0mnchPaWk-F9anvInjBBRbGXJF7Z.webp",
    "https://emvyjhleyhimxxdqybsn.supabase.co/storage/v1/object/public/avatars/1MOcMHPqN0hFbkpoqjdirZl4jgI5VFVqo.webp",
    "https://emvyjhleyhimxxdqybsn.supabase.co/storage/v1/object/public/avatars/198aosLkzeCyglhaKy5vPMeWktSJhFui_.webp",
    "https://emvyjhleyhimxxdqybsn.supabase.co/storage/v1/object/public/avatars/1ZYyoo8gUHeugXIa5ciA6pJySe3OPdkNB.webp",
];

pub const DEFAULT_AVATAR: &str = ALL_AVATARS[0];

/// Finds an avatar by its URL across all categories.
pub fn get_avatar_by_url(url: &str) -> Option<&'static Avatar> {
    for cat in AVATAR_CATEGORIES {
        if let Some(av) = cat.avatars.iter().find(|a| a.url == url) {
            return Some(av);
        }
    }
    None
}

/// Finds an avatar category by its ID.
pub fn get_avatar_category(id: &str) -> Option<&'static AvatarCategory> {
    AVATAR_CATEGORIES.iter().find(|c| c.id == id)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_avatar_counts() {
        assert_eq!(AVATAR_CATEGORIES.len(), 5);
        let total_nested: usize = AVATAR_CATEGORIES.iter().map(|c| c.avatars.len()).sum();
        assert_eq!(ALL_AVATARS.len(), total_nested);
    }

    /// The flat list holds every picture of the sets once; it starts with the classic smiles (a new profile gets one), so its order is not the sets' order.
    #[test]
    fn test_avatar_urls_match() {
        let mut nested: Vec<&str> = AVATAR_CATEGORIES.iter().flat_map(|c| c.avatars.iter().map(|a| a.url)).collect();
        let mut flat: Vec<&str> = ALL_AVATARS.to_vec();
        nested.sort_unstable();
        flat.sort_unstable();
        assert_eq!(nested, flat);
        assert!(ALL_AVATARS[0].ends_with("classic-red-smile.webp"));
    }

    #[test]
    fn test_avatar_lookups() {
        let default_av = get_avatar_by_url(DEFAULT_AVATAR);
        assert!(default_av.is_some());
        assert_eq!(default_av.unwrap().name, "Red Smile");

        assert!(get_avatar_by_url("https://invalid.url").is_none());

        let one_piece = get_avatar_category("one-piece");
        assert!(one_piece.is_some());
        assert_eq!(one_piece.unwrap().name, "One Piece");
        assert_eq!(one_piece.unwrap().avatars.len(), 11);
    }
}

/// Where the pictures are kept: a public bucket (WebP, 512 px, cached for a year), not the Google Drive links the app used first.
pub const AVATAR_BASE: &str = "https://emvyjhleyhimxxdqybsn.supabase.co/storage/v1/object/public/avatars/";
const DRIVE_BASE: &str = "https://lh3.googleusercontent.com/d/";

/// The id a stored picture address stands for: the file name in the bucket, or the id in an old Drive address.
pub fn avatar_id(url: &str) -> Option<String> {
    if let Some(rest) = url.strip_prefix(AVATAR_BASE) {
        return Some(rest.trim_end_matches(".webp").to_string());
    }
    url.strip_prefix(DRIVE_BASE).map(|id| id.to_string())
}

/// The bucket address of a picture id.
pub fn avatar_address(id: &str) -> String {
    format!("{AVATAR_BASE}{id}.webp")
}

/// Brings an address saved before the move (Google Drive) to the bucket, so an old profile loads fast too. Other addresses stay as they are.
pub fn fast_avatar(url: &str) -> String {
    match url.strip_prefix(DRIVE_BASE) {
        Some(id) if AVATAR_CATEGORIES.iter().flat_map(|c| c.avatars.iter()).any(|a| a.url.ends_with(&format!("/{id}.webp"))) => avatar_address(id),
        _ => url.to_string(),
    }
}

/// The six plain faces (red, dark blue, yellow, green, purple, pink), the pictures a new profile starts with.
pub const SMILE_IDS: [&str; 6] = ["classic-red-smile", "classic-dark-grey-smile", "classic-yellow-smile", "classic-green-smile", "classic-purple-smile", "classic-pink-smile"];

/// One of the six faces at random, as the address of its picture.
pub fn random_smile() -> String {
    let i = ((js_sys::Math::random() * SMILE_IDS.len() as f64) as usize).min(SMILE_IDS.len() - 1);
    avatar_address(SMILE_IDS[i])
}

/// Whether a Kids profile may have this picture: a picture of the sets made for Kids (the plain faces and the classics).
pub fn kids_ok(url: &str) -> bool {
    AVATAR_CATEGORIES.iter().filter(|c| c.kids).flat_map(|c| c.avatars.iter()).any(|a| a.url == url)
}
