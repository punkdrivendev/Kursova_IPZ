struct file_node {
    name: String,
    size: u64,
    category: String,
}

struct file_statistic {
    category: String,
    file_count: usize,
    total_size: u64,
    percentage: f64,
}

struct system_statistic {
    file_stat: Vec<file_statistic>,
    total_disk_space: u64,
    free_disk_space: u64,
    used_disk_space: u64,
}

struct StatService;

impl StatService {
    fn group_by_categories(nodes: Vec<file_node>) -> Vec<file_statistic> {
        todo!()
    }

    fn calculate_percentage() {
        todo!()
    }
}