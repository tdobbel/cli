const std = @import("std");

const reset = "\u{001b}[m";
const bold = "\u{001b}[1m";
const green = "\u{001b}[32m";
const yellow = "\u{001b}[33m";
const blue = "\u{001b}[34m";
const cyan = "\u{001b}[36m";

const User = struct {
    running: usize,
    pending: usize,
    partitions: std.StringHashMap(void),

    pub fn init(allocator: std.mem.Allocator) User {
        return User{
            .running = 0,
            .pending = 0,
            .partitions = std.StringHashMap(void).init(allocator),
        };
    }

    pub fn deinit(self: *User) void {
        self.partitions.deinit();
    }
};

const LineInfo = struct {
    user: []const u8,
    running: bool,
    partition: []const u8,
    job_id: []const u8,

    pub fn parse(line: []const u8) !LineInfo {
        var it = std.mem.tokenizeAny(u8, line, " ");
        var user: ?[]const u8 = null;
        var running: bool = false;
        var partition: ?[]const u8 = null;
        var job_id: ?[]const u8 = null;
        var i: usize = 0;
        while (it.next()) |elem| : (i += 1) {
            switch (i) {
                0 => {
                    user = elem;
                },
                1 => {
                    running = std.mem.eql(u8, elem, "R");
                },
                2 => {
                    partition = elem;
                },
                3 => {
                    job_id = elem;
                },
                else => {
                    unreachable;
                },
            }
        }
        return LineInfo{
            .user = user.?,
            .running = running,
            .partition = partition.?,
            .job_id = job_id.?,
        };
    }
};

pub fn free_users(queue: *std.StringHashMap(User)) void {
    var it = queue.iterator();
    while (it.next()) |entry| {
        entry.value_ptr.deinit();
    }
    queue.deinit();
}

pub fn parse_job_id(job_id: []const u8) !usize {
    var i: usize = 0;
    while (i < job_id.len and job_id[i] != '[') : (i += 1) {}
    if (i == job_id.len) {
        return 1;
    }
    var start: usize = i + 1;
    i = start;
    while (job_id[i] != '-') : (i += 1) {}
    const start_id = try std.fmt.parseInt(usize, job_id[start..i], 10);
    start = i + 1;
    i = start;
    while (job_id[i] >= '0' and job_id[i] <= '9') : (i += 1) {}
    const end_id = try std.fmt.parseInt(usize, job_id[start..i], 10);
    return end_id - start_id + 1;
}

pub fn less_than(queue: *std.StringHashMap(User), a: *[]const u8, b: *[]const u8) bool {
    const user_a = queue.get(a.*).?;
    const user_b = queue.get(b.*).?;
    return (user_a.running + user_a.pending) > (user_b.running + user_b.pending);
}

pub fn get_sorted_partitions(allocator: std.mem.Allocator, partition_set: *const std.StringHashMap(void)) ![]const u8 {
    var cntr: usize = 0;
    var it = partition_set.iterator();
    while (it.next()) |entry| {
        cntr += entry.key_ptr.len + 1;
    }
    var result = try allocator.alloc(u8, cntr);
    it = partition_set.iterator();
    var offset: usize = 0;
    while (it.next()) |entry| {
        const partition = entry.key_ptr;
        const size = partition.len;
        @memcpy(result[offset .. offset + size], partition.*);
        offset += size;
        result[offset] = ',';
        offset += 1;
    }
    return result[0 .. offset - 1];
}

pub fn main() !void {
    const allocator = std.heap.c_allocator;
    var queue = std.StringHashMap(User).init(allocator);
    defer free_users(&queue);
    var argv = std.ArrayList([]const u8).init(allocator);
    defer argv.deinit();
    var queue_size: usize = 0;
    try argv.appendSlice(&[_][]const u8{ "squeue", "--noheader", "-o %.20u %t %P %i" });
    var msg_end: ?[]const u8 = null;
    if (std.os.argv.len == 2) {
        try argv.append(try std.fmt.allocPrint(allocator, "--partition={s}", .{std.os.argv[1]}));
        msg_end = try std.fmt.allocPrint(allocator, "partition {s}", .{std.os.argv[1]});
    } else {
        msg_end = "the queue";
    }
    const result = try std.process.Child.run(.{
        .allocator = allocator,
        .argv = argv.items,
        .max_output_bytes = 512 * 1024,
    });
    defer allocator.free(result.stdout);
    var it = std.mem.tokenizeAny(u8, result.stdout, "\n");
    while (it.next()) |line| {
        const trimmed = std.mem.trimLeft(u8, line, " ");
        const line_info = try LineInfo.parse(trimmed);
        const entry = try queue.getOrPut(line_info.user);
        if (!entry.found_existing) {
            entry.value_ptr.* = User.init(allocator);
        }
        var user = entry.value_ptr;
        try user.partitions.put(line_info.partition, {});
        if (line_info.running) {
            user.running += 1;
            queue_size += 1;
        } else {
            const n_pending = try parse_job_id(line_info.job_id);
            user.pending += n_pending;
            queue_size += n_pending;
        }
    }
    if (queue_size == 0) {
        std.debug.print("🥳🎉 There are no jobs in {s} 🎉🥳", .{msg_end.?});
        return;
    }
    var user_names = try allocator.alloc(*[]const u8, queue.count());
    defer allocator.free(user_names);
    var queue_it = queue.iterator();
    var i: usize = 0;
    while (queue_it.next()) |entry| : (i += 1) {
        user_names[i] = entry.key_ptr;
    }
    std.mem.sort(*[]const u8, user_names, &queue, less_than);
    std.debug.print("There are {s}{}{s} jobs in {s}\n", .{ bold, queue_size, reset, msg_end.? });
    for (user_names) |user_name| {
        const user = queue.get(user_name.*).?;
        const partition_list = try get_sorted_partitions(allocator, &user.partitions);
        defer allocator.free(partition_list);
        std.debug.print("-> {s}{s:<12}{s}: ", .{ blue, user_name.*, reset });
        std.debug.print("{s}{s}{d:>4}{s} running, ", .{ green, bold, user.running, reset });
        std.debug.print("{s}{s}{d:>4}{s} pending  ", .{ yellow, bold, user.pending, reset });
        std.debug.print("({s}{s}{s}{s})\n", .{ cyan, bold, partition_list, reset });
    }
}
