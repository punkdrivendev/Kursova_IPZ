use sysinfo::{Disk, Disks};

pub struct SystemInfoService {
    disks: Disks,
}

impl SystemInfoService {
    pub fn new() -> Self {
        Self {
            disks: Disks::new_with_refreshed_list(),
        }
    }

    pub fn get_available_disks(&self) -> &[Disk] {
        self.disks.list()
    }

    pub fn disk_selection(&self, selection: usize) -> Option<&Disk> {
        self.disks.list().get(selection)
    }

    pub fn get_total_space(selected_disk: &Disk) -> u64 {
        selected_disk.total_space()
    }

    pub fn get_free_space(selected_disk: &Disk) -> u64 {
        selected_disk.available_space()
    }

    pub fn get_used_space(total_space: u64, free_space: u64) -> u64 {
        total_space - free_space
    }
}