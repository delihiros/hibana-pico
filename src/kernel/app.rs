use crate::{
    choreography::protocol::{
        MEM_LEASE_NONE, MemBorrow, MemCommit, MemRelease, MemRights, StdinRequest,
    },
    kernel::wasi::MemoryLeaseError,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct AppId(u8);

impl AppId {
    pub const fn new(raw: u8) -> Self {
        Self(raw)
    }

    pub const fn raw(self) -> u8 {
        self.0
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AppScopeError {
    BadApp,
    TableFull,
    InvalidHandle,
    UnknownHandle,
    GenerationMismatch,
    RightsMismatch,
    Memory(MemoryLeaseError),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct AppStreamHandle {
    app: AppId,
    handle: u8,
    generation: u8,
    rights: MemRights,
}

impl AppStreamHandle {
    pub const fn app(&self) -> AppId {
        self.app
    }

    pub const fn handle(&self) -> u8 {
        self.handle
    }

    pub const fn generation(&self) -> u8 {
        self.generation
    }

    pub const fn rights(&self) -> MemRights {
        self.rights
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct StreamEntry {
    app: AppId,
    handle: u8,
    generation: u8,
    rights: MemRights,
    open: bool,
}

pub struct AppStreamTable<const N: usize> {
    entries: [Option<StreamEntry>; N],
}

impl<const N: usize> AppStreamTable<N> {
    pub const fn new() -> Self {
        Self { entries: [None; N] }
    }

    pub fn open(
        &mut self,
        app: AppId,
        rights: MemRights,
    ) -> Result<AppStreamHandle, AppScopeError> {
        let index = self
            .entries
            .iter()
            .position(Option::is_none)
            .ok_or(AppScopeError::TableFull)?;
        let handle = (index + 1) as u8;
        let entry = StreamEntry {
            app,
            handle,
            generation: 1,
            rights,
            open: true,
        };
        self.entries[index] = Some(entry);
        Ok(AppStreamHandle {
            app,
            handle,
            generation: entry.generation,
            rights,
        })
    }

    pub fn validate(
        &self,
        app: AppId,
        handle: AppStreamHandle,
        required: MemRights,
    ) -> Result<(), AppScopeError> {
        if app != handle.app {
            return Err(AppScopeError::BadApp);
        }
        let entry = self.get(handle.handle)?;
        if entry.app != app {
            return Err(AppScopeError::BadApp);
        }
        if !entry.open {
            return Err(AppScopeError::UnknownHandle);
        }
        if entry.generation != handle.generation {
            return Err(AppScopeError::GenerationMismatch);
        }
        if entry.rights != required {
            return Err(AppScopeError::RightsMismatch);
        }
        Ok(())
    }

    pub fn close(&mut self, app: AppId, handle: AppStreamHandle) -> Result<(), AppScopeError> {
        self.validate(app, handle, handle.rights)?;
        let entry = self.get_mut(handle.handle)?;
        entry.open = false;
        entry.generation = entry.generation.wrapping_add(1).max(1);
        Ok(())
    }

    fn get(&self, handle: u8) -> Result<StreamEntry, AppScopeError> {
        if handle == 0 {
            return Err(AppScopeError::InvalidHandle);
        }
        self.entries
            .get(handle as usize - 1)
            .and_then(|entry| *entry)
            .ok_or(AppScopeError::UnknownHandle)
    }

    fn get_mut(&mut self, handle: u8) -> Result<&mut StreamEntry, AppScopeError> {
        if handle == 0 {
            return Err(AppScopeError::InvalidHandle);
        }
        self.entries
            .get_mut(handle as usize - 1)
            .and_then(Option::as_mut)
            .ok_or(AppScopeError::UnknownHandle)
    }
}

impl<const N: usize> Default for AppStreamTable<N> {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct AppLeaseGrant {
    app: AppId,
    lease_id: u8,
    len: u8,
    epoch: u32,
    rights: MemRights,
}

impl AppLeaseGrant {
    pub const fn lease_id(&self) -> u8 {
        self.lease_id
    }

    pub const fn app(&self) -> AppId {
        self.app
    }
}

pub struct AppLeaseTable<const N: usize> {
    memory_len: u32,
    slots: [Option<AppLeaseGrant>; N],
}

impl<const N: usize> AppLeaseTable<N> {
    pub const fn new(memory_len: u32) -> Self {
        Self {
            memory_len,
            slots: [None; N],
        }
    }

    pub fn grant_read(
        &mut self,
        app: AppId,
        borrow: MemBorrow,
    ) -> Result<AppLeaseGrant, AppScopeError> {
        self.grant(app, borrow, MemRights::Read)
    }

    pub fn grant_write(
        &mut self,
        app: AppId,
        borrow: MemBorrow,
    ) -> Result<AppLeaseGrant, AppScopeError> {
        self.grant(app, borrow, MemRights::Write)
    }

    pub fn validate_read(&self, app: AppId, lease_id: u8, len: usize) -> Result<(), AppScopeError> {
        let grant = self.get(app, lease_id)?;
        if grant.rights != MemRights::Read {
            return Err(AppScopeError::RightsMismatch);
        }
        if len > grant.len as usize {
            return Err(AppScopeError::Memory(MemoryLeaseError::LengthExceeded));
        }
        Ok(())
    }

    pub fn validate_write_request(
        &self,
        app: AppId,
        request: StdinRequest,
    ) -> Result<(), AppScopeError> {
        let grant = self.get(app, request.lease_id())?;
        if grant.rights != MemRights::Write {
            return Err(AppScopeError::RightsMismatch);
        }
        if request.max_len() > grant.len {
            return Err(AppScopeError::Memory(MemoryLeaseError::LengthExceeded));
        }
        Ok(())
    }

    pub fn commit(&self, app: AppId, commit: MemCommit) -> Result<(), AppScopeError> {
        let grant = self.get(app, commit.lease_id())?;
        if grant.rights != MemRights::Write {
            return Err(AppScopeError::RightsMismatch);
        }
        if commit.written() > grant.len {
            return Err(AppScopeError::Memory(MemoryLeaseError::LengthExceeded));
        }
        Ok(())
    }

    pub fn release(&mut self, app: AppId, release: MemRelease) -> Result<(), AppScopeError> {
        if release.lease_id() == MEM_LEASE_NONE {
            return Err(AppScopeError::InvalidHandle);
        }
        for slot in &mut self.slots {
            if let Some(grant) = slot
                && grant.app == app
                && grant.lease_id == release.lease_id()
            {
                *slot = None;
                return Ok(());
            }
        }
        Err(AppScopeError::UnknownHandle)
    }

    fn grant(
        &mut self,
        app: AppId,
        borrow: MemBorrow,
        rights: MemRights,
    ) -> Result<AppLeaseGrant, AppScopeError> {
        if borrow.len() == 0 {
            return Err(AppScopeError::Memory(MemoryLeaseError::Empty));
        }
        let end = borrow
            .ptr()
            .checked_add(borrow.len() as u32)
            .ok_or(AppScopeError::Memory(MemoryLeaseError::OutOfBounds))?;
        if end > self.memory_len {
            return Err(AppScopeError::Memory(MemoryLeaseError::OutOfBounds));
        }
        let index = self
            .slots
            .iter()
            .position(Option::is_none)
            .ok_or(AppScopeError::TableFull)?;
        let grant = AppLeaseGrant {
            app,
            lease_id: self.allocate_lease_id(app)?,
            len: borrow.len(),
            epoch: borrow.epoch(),
            rights,
        };
        self.slots[index] = Some(grant);
        Ok(grant)
    }

    fn allocate_lease_id(&self, app: AppId) -> Result<u8, AppScopeError> {
        for candidate in 1..=u8::MAX {
            if self
                .slots
                .iter()
                .flatten()
                .all(|grant| grant.app != app || grant.lease_id != candidate)
            {
                return Ok(candidate);
            }
        }
        Err(AppScopeError::TableFull)
    }

    fn get(&self, app: AppId, lease_id: u8) -> Result<AppLeaseGrant, AppScopeError> {
        if lease_id == MEM_LEASE_NONE {
            return Err(AppScopeError::InvalidHandle);
        }
        for grant in self.slots.iter().flatten() {
            if grant.app == app && grant.lease_id == lease_id {
                return Ok(*grant);
            }
        }
        Err(AppScopeError::UnknownHandle)
    }
}

impl<const N: usize> Default for AppLeaseTable<N> {
    fn default() -> Self {
        Self::new(0)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct AppImageSlot {
    app: AppId,
    slot: u8,
    generation: u32,
}

pub struct AppImageRegistry<const N: usize> {
    slots: [Option<AppImageSlot>; N],
}

impl<const N: usize> AppImageRegistry<N> {
    pub const fn new() -> Self {
        Self { slots: [None; N] }
    }

    pub fn install(&mut self, app: AppId, slot: u8, generation: u32) -> Result<(), AppScopeError> {
        let index = self
            .slots
            .iter()
            .position(|entry| entry.is_none_or(|entry| entry.slot == slot))
            .ok_or(AppScopeError::TableFull)?;
        self.slots[index] = Some(AppImageSlot {
            app,
            slot,
            generation,
        });
        Ok(())
    }

    pub fn activate(&self, app: AppId, slot: u8, generation: u32) -> Result<(), AppScopeError> {
        let image = self
            .slots
            .iter()
            .flatten()
            .find(|entry| entry.slot == slot)
            .ok_or(AppScopeError::UnknownHandle)?;
        if image.app != app {
            return Err(AppScopeError::BadApp);
        }
        if image.generation != generation {
            return Err(AppScopeError::GenerationMismatch);
        }
        Ok(())
    }
}

impl<const N: usize> Default for AppImageRegistry<N> {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::{AppId, AppImageRegistry, AppLeaseTable, AppScopeError, AppStreamTable};
    use crate::choreography::protocol::{
        MemBorrow, MemCommit, MemRelease, MemRights, StdinRequest,
    };

    #[test]
    fn app_stream_table_rejects_cross_app_and_stale_generation() {
        let app0 = AppId::new(0);
        let app1 = AppId::new(1);
        let mut streams: AppStreamTable<2> = AppStreamTable::new();
        let handle = streams.open(app0, MemRights::Read).expect("open stream");
        streams
            .validate(app0, handle, MemRights::Read)
            .expect("app0 stream");
        assert_eq!(
            streams.validate(app1, handle, MemRights::Read),
            Err(AppScopeError::BadApp)
        );

        streams.close(app0, handle).expect("close stream");
        assert_eq!(
            streams.validate(app0, handle, MemRights::Read),
            Err(AppScopeError::UnknownHandle)
        );
    }

    #[test]
    fn wasip1_stream_close_invalid_reuse_smoke() {
        let app = AppId::new(0);
        let mut streams: AppStreamTable<1> = AppStreamTable::new();
        let handle = streams.open(app, MemRights::Write).expect("open stream");
        streams
            .validate(app, handle, MemRights::Write)
            .expect("stream is valid before close");
        streams.close(app, handle).expect("close stream");

        assert_eq!(
            streams.validate(app, handle, MemRights::Write),
            Err(AppScopeError::UnknownHandle)
        );
    }

    #[test]
    fn app_lease_table_scopes_lease_ids_per_app() {
        let app0 = AppId::new(0);
        let app1 = AppId::new(1);
        let mut leases: AppLeaseTable<4> = AppLeaseTable::new(4096);
        let read0 = leases
            .grant_read(app0, MemBorrow::new(1024, 8, 1))
            .expect("grant read app0");
        let read1 = leases
            .grant_read(app1, MemBorrow::new(2048, 8, 1))
            .expect("grant read app1");
        assert_eq!(read0.lease_id(), read1.lease_id());
        leases
            .validate_read(app0, read0.lease_id(), 8)
            .expect("read app0");
        assert_eq!(
            leases.validate_read(app1, read0.lease_id(), 9),
            Err(AppScopeError::Memory(
                crate::kernel::wasi::MemoryLeaseError::LengthExceeded
            ))
        );
        leases
            .release(app0, MemRelease::new(read0.lease_id()))
            .expect("release app0");
        leases
            .validate_read(app1, read1.lease_id(), 8)
            .expect("app1 unaffected");
    }

    #[test]
    fn app_lease_table_rejects_cross_app_write_commit() {
        let app0 = AppId::new(0);
        let app1 = AppId::new(1);
        let mut leases: AppLeaseTable<2> = AppLeaseTable::new(4096);
        let grant = leases
            .grant_write(app0, MemBorrow::new(1024, 24, 1))
            .expect("grant write");
        let request = StdinRequest::new_with_lease(grant.lease_id(), 24).expect("request");
        leases
            .validate_write_request(app0, request)
            .expect("app0 request");
        assert_eq!(
            leases.commit(app1, MemCommit::new(grant.lease_id(), 8)),
            Err(AppScopeError::UnknownHandle)
        );
    }

    #[test]
    fn app_image_registry_scopes_slots_and_generations() {
        let app0 = AppId::new(0);
        let app1 = AppId::new(1);
        let mut images: AppImageRegistry<2> = AppImageRegistry::new();
        images.install(app0, 0, 7).expect("install app0 image");
        images.activate(app0, 0, 7).expect("activate app0 image");
        assert_eq!(images.activate(app1, 0, 7), Err(AppScopeError::BadApp));
        assert_eq!(
            images.activate(app0, 0, 8),
            Err(AppScopeError::GenerationMismatch)
        );
    }
}
