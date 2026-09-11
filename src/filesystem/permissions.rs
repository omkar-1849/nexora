use crate::error::{EnvError, EnvResult};
use crate::filesystem::user::ROOT_USER_ID;
use crate::filesystem::FileMetadata;

pub const READ: u16 = 0b100;
pub const WRITE: u16 = 0b010;
pub const EXECUTE: u16 = 0b001;

pub const OWNER_READ: u16 = READ << 6;
pub const OWNER_WRITE: u16 = WRITE << 6;
pub const OWNER_EXECUTE: u16 = EXECUTE << 6;

pub const GROUP_READ: u16 = READ << 3;
pub const GROUP_WRITE: u16 = WRITE << 3;
pub const GROUP_EXECUTE: u16 = EXECUTE << 3;

pub const OTHER_READ: u16 = READ;
pub const OTHER_WRITE: u16 = WRITE;
pub const OTHER_EXECUTE: u16 = EXECUTE;

pub const DEFAULT_FILE_PERMISSIONS: u16 = OWNER_READ | OWNER_WRITE | GROUP_READ | OTHER_READ;

pub const DEFAULT_DIRECTORY_PERMISSIONS: u16 = OWNER_READ
    | OWNER_WRITE
    | OWNER_EXECUTE
    | GROUP_READ
    | GROUP_EXECUTE
    | OTHER_READ
    | OTHER_EXECUTE;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Permission {
    Read,
    Write,
    Execute,
}

pub fn check_permission(
    metadata: &FileMetadata,
    user_id: u32,
    permission: Permission,
) -> EnvResult<()> {
    // Root has unrestricted filesystem access.
    if user_id == ROOT_USER_ID {
        return Ok(());
    }

    let bit = match permission {
        Permission::Read => READ,
        Permission::Write => WRITE,
        Permission::Execute => EXECUTE,
    };

    let permission_bits = if user_id == metadata.owner_id {
        (metadata.permissions >> 6) & 0b111
    } else {
        metadata.permissions & 0b111
    };

    if permission_bits & bit != 0 {
        Ok(())
    } else {
        Err(EnvError::PermissionDenied(format!(
            "User {} does not have {:?} permission.",
            user_id, permission
        )))
    }
}
