//! Localized user-facing resources for MiniRust.
//!
//! Text lives in `locales/<language>.json`, keyed by a stable identifier. Code refers to
//! [`Key`] variants, so a missing or misspelled resource is a compile error or a test
//! failure rather than a silent fallback.

use std::collections::HashMap;
use std::sync::OnceLock;

const ENGLISH: &str = include_str!("../locales/en.json");
const VIETNAMESE: &str = include_str!("../locales/vi.json");

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Locale {
    Vi,
    En,
}

impl Locale {
    pub const DEFAULT: Self = Self::Vi;

    pub fn parse(value: &str) -> Self {
        if value.trim().eq_ignore_ascii_case("en") {
            Self::En
        } else {
            Self::Vi
        }
    }

    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Vi => "vi",
            Self::En => "en",
        }
    }
}

macro_rules! keys {
    ($($variant:ident => $name:literal),* $(,)?) => {
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
        pub enum Key {
            $($variant),*
        }

        impl Key {
            pub const ALL: &'static [Self] = &[$(Self::$variant),*];

            pub const fn as_str(self) -> &'static str {
                match self {
                    $(Self::$variant => $name),*
                }
            }
        }
    };
}

keys! {
    AdminActionCancel => "admin.action.cancel",
    AdminActionClose => "admin.action.close",
    AdminActionDelete => "admin.action.delete",
    AdminActionEdit => "admin.action.edit",
    AdminActionYou => "admin.action.you",
    AdminActions => "admin.actions",
    AdminCreateActiveNote => "admin.create.active_note",
    AdminCreateCloseLabel => "admin.create.close_label",
    AdminCreateEyebrow => "admin.create.eyebrow",
    AdminCreateOpen => "admin.create.open",
    AdminCreateSendInvite => "admin.create.send_invite",
    AdminCreateTitle => "admin.create.title",
    AdminDeleteCloseLabel => "admin.delete.close_label",
    AdminDeleteConfirm => "admin.delete.confirm",
    AdminDeletePrefix => "admin.delete.prefix",
    AdminDeleteSuffix => "admin.delete.suffix",
    AdminDeleteTitle => "admin.delete.title",
    AdminEditCanSignIn => "admin.edit.can_sign_in",
    AdminEditCannotSignIn => "admin.edit.cannot_sign_in",
    AdminEditCloseLabel => "admin.edit.close_label",
    AdminEditDisableSignInAria => "admin.edit.disable_sign_in_aria",
    AdminEditEyebrow => "admin.edit.eyebrow",
    AdminEditRoleLockedHint => "admin.edit.role_locked_hint",
    AdminEditSave => "admin.edit.save",
    AdminEmpty => "admin.empty",
    AdminEyebrow => "admin.eyebrow",
    AdminFilterPremiumAll => "admin.filter.premium.all",
    AdminFilterPremiumLabel => "admin.filter.premium.label",
    AdminFilterPremiumOff => "admin.filter.premium.off",
    AdminFilterRoleAll => "admin.filter.role.all",
    AdminFilterRoleLabel => "admin.filter.role.label",
    AdminFilterStatusAll => "admin.filter.status.all",
    AdminFilterStatusLabel => "admin.filter.status.label",
    AdminLoading => "admin.loading",
    AdminPaginationAll => "admin.pagination.all",
    AdminPaginationItems => "admin.pagination.items",
    AdminPaginationOf => "admin.pagination.of",
    AdminPaginationPage => "admin.pagination.page",
    AdminPremiumExpiryAria => "admin.premium.expiry_aria",
    AdminPremiumExpiryHint => "admin.premium.expiry_hint",
    AdminPremiumExpiryLabel => "admin.premium.expiry_label",
    AdminPremiumExpiryPicker => "admin.premium.expiry_picker",
    AdminPremiumGrantCreate => "admin.premium.grant_create",
    AdminPremiumGrantEdit => "admin.premium.grant_edit",
    AdminPremiumInactive => "admin.premium.inactive",
    AdminPremiumUnlimited => "admin.premium.unlimited",
    AdminPremiumUntil => "admin.premium.until",
    AdminRole => "admin.role",
    AdminSearchLabel => "admin.search.label",
    AdminSearchPlaceholder => "admin.search.placeholder",
    AdminSignInAccess => "admin.sign_in_access",
    AdminSignInDisabled => "admin.sign_in_disabled",
    AdminStatus => "admin.status",
    AdminTitle => "admin.title",
    AdminToastCreated => "admin.toast.created",
    AdminToastCreatedInvite => "admin.toast.created_invite",
    AdminToastDeleted => "admin.toast.deleted",
    AdminToastSaved => "admin.toast.saved",
    CommonActive => "common.active",
    CommonLoading => "common.loading",
    CommonLocked => "common.locked",
    CommonRoleAdmin => "common.role_admin",
    CommonRoleUser => "common.role_user",
    CommonSortAscending => "common.sort_ascending",
    CommonSortBy => "common.sort_by",
    CommonSortDescending => "common.sort_descending",
    AuthAlreadyRegistered => "auth.already_registered",
    AuthBackToRegister => "auth.back_to_register",
    AuthCheckEmail => "auth.check_email",
    AuthCodeRequested => "auth.code_requested",
    AuthCreateAccount => "auth.create_account",
    AuthEmailRegistered => "auth.email_registered",
    AuthGoToLogin => "auth.go_to_login",
    AuthInvalidLink => "auth.invalid_link",
    AuthLinkSent => "auth.link_sent",
    AuthLocalTestLink => "auth.local_test_link",
    AuthLoginEyebrow => "auth.login_eyebrow",
    AuthLoginHint => "auth.login_hint",
    AuthLoginTitle => "auth.login_title",
    AuthNewHere => "auth.new_here",
    AuthOpenVerificationLink => "auth.open_verification_link",
    AuthRegisterEyebrow => "auth.register_eyebrow",
    AuthRegisterHint => "auth.register_hint",
    AuthRegisterSubmit => "auth.register_submit",
    AuthRegisterTitle => "auth.register_title",
    AuthResendHint => "auth.resend_hint",
    AuthSendCode => "auth.send_code",
    AuthTryAgain => "auth.try_again",
    AuthVerificationCode => "auth.verification_code",
    AuthVerifyContinue => "auth.verify_continue",
    AuthVerifyEyebrow => "auth.verify_eyebrow",
    AuthVerifyTitle => "auth.verify_title",
    AuthVerifying => "auth.verifying",
    DashboardApplicationBody => "dashboard.application.body",
    DashboardApplicationTitle => "dashboard.application.title",
    DashboardComponents => "dashboard.components",
    DashboardDataBody => "dashboard.data.body",
    DashboardDataTitle => "dashboard.data.title",
    DashboardEyebrow => "dashboard.eyebrow",
    DashboardFrontendBody => "dashboard.frontend.body",
    DashboardFrontendTitle => "dashboard.frontend.title",
    DashboardGetStarted => "dashboard.get_started",
    DashboardIntro => "dashboard.intro",
    DashboardTransportBody => "dashboard.transport.body",
    DashboardUserAdmin => "dashboard.user_admin",
    DashboardWelcome => "dashboard.welcome",
    ErrorsAuthAccountLocked => "errors.auth.account_locked",
    ErrorsAuthAttemptsExceeded => "errors.auth.attempts_exceeded",
    ErrorsAuthBootstrapAdminConflict => "errors.auth.bootstrap_admin_conflict",
    ErrorsAuthCodeExpired => "errors.auth.code_expired",
    ErrorsAuthEmailAlreadyExists => "errors.auth.email_already_exists",
    ErrorsAuthEmailDeliveryUnavailable => "errors.auth.email_delivery_unavailable",
    ErrorsAuthInvalidCode => "errors.auth.invalid_code",
    ErrorsAuthInvalidEmail => "errors.auth.invalid_email",
    ErrorsAuthInvalidVerificationToken => "errors.auth.invalid_verification_token",
    ErrorsAuthSessionInvalid => "errors.auth.session_invalid",
    ErrorsAuthVerificationTokenExpired => "errors.auth.verification_token_expired",
    ErrorsAuthorizationForbidden => "errors.authorization.forbidden",
    ErrorsDependencyUnavailable => "errors.dependency.unavailable",
    ErrorsInternalUnexpected => "errors.internal.unexpected",
    ErrorsMenuInvalidIcon => "errors.menu.invalid_icon",
    ErrorsMenuInvalidName => "errors.menu.invalid_name",
    ErrorsMenuInvalidParent => "errors.menu.invalid_parent",
    ErrorsMenuInvalidPath => "errors.menu.invalid_path",
    ErrorsMenuNotFound => "errors.menu.not_found",
    ErrorsRateLimitExceeded => "errors.rate_limit.exceeded",
    ErrorsRequestBadRequest => "errors.request.bad_request",
    ErrorsRequestNotFound => "errors.request.not_found",
    ErrorsRequestPayloadTooLarge => "errors.request.payload_too_large",
    ErrorsUploadAvatarDirectoryInvalid => "errors.upload.avatar_directory_invalid",
    ErrorsUploadFileNameRequired => "errors.upload.file_name_required",
    ErrorsUploadFileTooLarge => "errors.upload.file_too_large",
    ErrorsUploadUnsupportedAvatarFormat => "errors.upload.unsupported_avatar_format",
    ErrorsUserCannotChangeOwnRole => "errors.user.cannot_change_own_role",
    ErrorsUserEmailAlreadyExists => "errors.user.email_already_exists",
    ErrorsUserInvalidAvatarUrl => "errors.user.invalid_avatar_url",
    ErrorsUserInvalidEmail => "errors.user.invalid_email",
    ErrorsUserInvalidFullName => "errors.user.invalid_full_name",
    ErrorsUserInvalidLocale => "errors.user.invalid_locale",
    ErrorsUserInvalidPremiumExpiry => "errors.user.invalid_premium_expiry",
    ErrorsUserInvalidRole => "errors.user.invalid_role",
    ErrorsUserNotFound => "errors.user.not_found",
    ErrorsUserProtected => "errors.user.protected",
    ErrorsValidationMessageRequired => "errors.validation.message_required",
    ErrorsValidationMessageTooLong => "errors.validation.message_too_long",
    MenusAdminNote => "menus.admin_note",
    MenusAdministrators => "menus.administrators",
    MenusEmpty => "menus.empty",
    MenusFullAccess => "menus.full_access",
    MenusHidden => "menus.hidden",
    MenusIntro => "menus.intro",
    MenusNormalUsers => "menus.normal_users",
    MenusPath => "menus.path",
    MenusSaved => "menus.saved",
    NavMenuPermissions => "nav.menu_permissions",
    NavProfile => "nav.profile",
    NavSignOut => "nav.sign_out",
    PremiumBadgeTitle => "premium.badge_title",
    ProfileAvatar => "profile.avatar",
    ProfileAvatarHint => "profile.avatar_hint",
    ProfileDangerHint => "profile.danger_hint",
    ProfileDangerZone => "profile.danger_zone",
    ProfileEyebrow => "profile.eyebrow",
    ProfileFullName => "profile.full_name",
    ProfileLockAccount => "profile.lock_account",
    ProfileRole => "profile.role",
    ProfileSave => "profile.save",
    ProfileSaved => "profile.saved",
    ProfileSaving => "profile.saving",
    ProfileSection => "profile.section",
    ProfileSectionHint => "profile.section_hint",
    ProfileStatus => "profile.status",
    ProfileTitle => "profile.title",
    SidebarEmpty => "sidebar.empty",
}

/// Returns the localized text for `key`, falling back to English and then to the key
/// itself so a lookup can never panic at runtime.
pub fn text(locale: Locale, key: Key) -> &'static str {
    let key_name = key.as_str();

    table(locale)
        .get(key_name)
        .or_else(|| table(Locale::En).get(key_name))
        .map(String::as_str)
        .unwrap_or(key_name)
}

fn table(locale: Locale) -> &'static HashMap<String, String> {
    static ENGLISH_TABLE: OnceLock<HashMap<String, String>> = OnceLock::new();
    static VIETNAMESE_TABLE: OnceLock<HashMap<String, String>> = OnceLock::new();

    match locale {
        Locale::Vi => VIETNAMESE_TABLE.get_or_init(|| parse(VIETNAMESE)),
        Locale::En => ENGLISH_TABLE.get_or_init(|| parse(ENGLISH)),
    }
}

fn parse(resource: &str) -> HashMap<String, String> {
    serde_json::from_str(resource).unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resource_files_parse() {
        assert!(!parse(ENGLISH).is_empty());
        assert!(!parse(VIETNAMESE).is_empty());
    }

    #[test]
    fn every_key_has_both_translations() {
        for key in Key::ALL {
            let name = key.as_str();

            assert!(
                table(Locale::En).contains_key(name),
                "en.json is missing {name}"
            );
            assert!(
                table(Locale::Vi).contains_key(name),
                "vi.json is missing {name}"
            );
        }
    }

    #[test]
    fn resource_files_do_not_declare_unknown_keys() {
        let known = Key::ALL.iter().map(|key| key.as_str()).collect::<Vec<_>>();

        for name in table(Locale::En).keys().chain(table(Locale::Vi).keys()) {
            assert!(
                known.contains(&name.as_str()),
                "{name} is not declared as a Key variant"
            );
        }
    }

    #[test]
    fn translations_differ_per_locale() {
        assert_eq!(text(Locale::Vi, Key::ProfileSave), "Lưu hồ sơ");
        assert_eq!(text(Locale::En, Key::ProfileSave), "Save profile");
    }

    #[test]
    fn unknown_locale_names_fall_back_to_vietnamese() {
        assert_eq!(Locale::parse("fr"), Locale::Vi);
        assert_eq!(Locale::parse("EN"), Locale::En);
    }
}
