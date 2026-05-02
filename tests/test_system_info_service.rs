use kursova_ipz::stat_services::system_info_service::SystemInfoService;

#[test]
fn test_get_used_space() {
    let result = SystemInfoService::get_used_space(1000, 300);

    assert_eq!(result, 700);
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
fn test_print_selected_disk() {
    let service = SystemInfoService::new();

    let selected_disk = service.disk_selection(0);

    match selected_disk {
        Some(disk) => {
            let total = SystemInfoService::get_total_space(disk);
            let free = SystemInfoService::get_free_space(disk);
            let used = SystemInfoService::get_used_space(total, free);

            println!("Selected disk:");
            println!("Name: {:?}", disk.name());
            println!("Mount point: {:?}", disk.mount_point());
            println!("Total space: {}", total);
            println!("Free space: {}", free);
            println!("Used space: {}", used);

            assert_eq!(used, total - free);
        }

        None => {
            println!("No disk found at index 0");
        }
    }
}