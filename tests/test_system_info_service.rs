use disk_analyzer::stat_services::system_info_service::SystemInfoService;

#[test]
fn test_load_disk_stats() {
    let mut service = SystemInfoService::new();

    let disks = service.get_available_disks();

    if disks.is_empty() {
        println!("No disks found");
        return;
    }

    let loaded = service.load_disk_stats(0);

    assert!(loaded);
    assert!(service.get_total_disk_space() > 0);

    let total = service.get_total_disk_space();
    let free = service.get_free_disk_space();
    let used = service.get_used_disk_space();

    assert_eq!(used, total - free);
}

#[test]
fn test_print_available_disks() {
    let service = SystemInfoService::new();

    let disks = service.get_available_disks();

    println!("Found disks: {}", disks.len());

    for (index, disk) in disks.iter().enumerate() {
        println!("Disk index: {}", index);
        println!("Name: {:?}", disk.name());
        println!("Mount point: {:?}", disk.mount_point());
        println!("Total space: {}", disk.total_space());
        println!("Free space: {}", disk.available_space());
        println!("Used space: {}", disk.total_space() - disk.available_space());
        println!("-------------------------");
    }
}

#[test]
fn test_selected_disk_stats() {
    let mut service = SystemInfoService::new();

    let disks = service.get_available_disks();

    if disks.is_empty() {
        println!("No disks found");
        return;
    }

    let loaded = service.load_disk_stats(0);

    assert!(loaded);

    let total = service.get_total_disk_space();
    let free = service.get_free_disk_space();
    let used = service.get_used_disk_space();

    println!("Selected disk stats:");
    println!("Total space: {}", total);
    println!("Free space: {}", free);
    println!("Used space: {}", used);

    assert!(total > 0);
    assert_eq!(used, total - free);
}