pub mod about;
pub mod allmaps;
pub mod change_password;
pub mod change_username;
pub mod index;
pub mod login;
pub mod map;
pub mod recent;
pub mod register;
pub mod replay;
pub mod search;
// pub mod search2;
pub mod upload_map;
pub mod upload_replay;
pub mod user;
pub mod viewer;

fn get_navbar_langmap(lang: bwcommon::LangData) -> serde_json::Value {
    if lang == bwcommon::LangData::Korean {
        serde_json::json!({
            "home": "홈으로",
            "search": "검색",
            "recent": "최신순",
            "about": "정보",
            "upload_map": "지도 업로드",
            "upload_replay": "다시보기 업로드",
            "login": "로그인",
            "register": "가입하기",
            "change_username": "사용자 이름 변경",
            "change_password": "비밀번호 변경",
            "logout": "로그 아웃",
        })
    } else {
        serde_json::json!({
            "home": "Home",
            "search": "Search",
            "recent": "Recent",
            "about": "About",
            "upload_map": "Upload Map",
            "upload_replay": "Upload Replay",
            "login": "Log in",
            "register": "Register",
            "change_username": "Change Username",
            "change_password": "Change Password",
            "logout": "Log out",
        })
    }
}
