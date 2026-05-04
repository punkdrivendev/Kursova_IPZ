use sysinfo::{Disk, Disks};

pub struct SystemInfoService {
    disks: Disks,
    total_disk_space: u64,
    free_disk_space: u64,
    used_disk_space: u64,
}

impl SystemInfoService {
    pub fn new() -> Self {
        Self {
            disks: Disks::new_with_refreshed_list(),
            total_disk_space: 0,
            free_disk_space: 0,
            used_disk_space: 0,
        }
    }

    pub fn get_available_disks(&self) -> &[Disk] {
        self.disks.list()
    }

    pub fn disk_selection(&self, selection: usize) -> Option<&Disk> {
        self.disks.list().get(selection)
    }

    pub fn load_disk_stats(&mut self, selection: usize) -> bool {
        if let Some(selected_disk) = self.disks.list().get(selection) {
            self.total_disk_space = selected_disk.total_space();
            self.free_disk_space = selected_disk.available_space();
            self.used_disk_space = self.total_disk_space - self.free_disk_space;

            true
        } else {
            false
        }
    }

    pub fn get_total_disk_space(&self) -> u64 {
        self.total_disk_space
    }

    pub fn get_free_disk_space(&self) -> u64 {
        self.free_disk_space
    }

    pub fn get_used_disk_space(&self) -> u64 {
        self.used_disk_space
    }
}