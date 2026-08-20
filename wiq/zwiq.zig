const std = @import("std");

const reset = "\u{001b}[m";
const bold = "\u{001b}[1m";
const green = "\u{001b}[32m";
const yellow = "\u{001b}[33m";
const blue = "\u{001b}[34m";
const cyan = "\u{001b}[36m";

const n_entry: usize = 4;
const max_partition: usize = 20;

const User = struct {
    running: usize,
    pending: usize,
    n_partition: usize,
    partitions: [max_partition]usize,

    pub fn add_partition(self: *User, p: usize) void {
        for (0..self.n_partition) |i| {
            if (p == self.partitions[i]) return;
        }
        self.partitions[self.n_partition] = p;
        self.n_partition += 1;
    }

    pub fn print(self: *const User, writer: *std.Io.Writer, user_name: []const u8, partition_names: [][]const u8) !void {
        try writer.print("-> {s}{s:<12}{s}: ", .{ blue, user_name, reset });
        try writer.print("{s}{s}{d:>4}{s} running, ", .{ green, bold, self.running, reset });
        try writer.print("{s}{s}{d:>4}{s} pending  ", .{ yellow, bold, self.pending, reset });
        try writer.print("({s}{s}", .{ cyan, bold });
        for (0..self.n_partition) |i| {
            const part_name = partition_names[self.partitions[i]];
            try writer.print("{s}", .{part_name});
            if (i < self.n_partition - 1) {
                try writer.print(", ", .{});
            }
        }
        try writer.print("{s})\n", .{reset});
        try writer.flush();
    }
};

const Queue = struct {
    allocator: std.mem.Allocator,
    users: std.StringHashMap(User),
    partitions: std.ArrayList([]const u8),
    total_jobs: usize,

    pub fn init(allocator: std.mem.Allocator) Queue {
        return Queue{
            .allocator = allocator,
            .users = std.StringHashMap(User).init(allocator),
            .partitions = .empty,
            .total_jobs = 0,
        };
    }

    pub fn partition_id(self: *Queue, partition: []const u8) !usize {
        for (self.partitions.items, 0..) |p, i| {
            if (std.mem.eql(u8, p, partition)) return i;
        }
        try self.partitions.append(self.allocator, try self.allocator.dupe(u8, partition));
        return self.partitions.items.len;
    }

    pub fn deinit(self: *Queue) void {
        self.users.deinit();
        self.partitions.deinit(self.allocator);
    }

    pub fn get_user(self: *Queue, name: []const u8) !*User {
        if (self.users.contains(name)) return self.users.getEntry(name).?.value_ptr;
        const key = try self.allocator.dupe(u8, name);
        const entry = try self.users.getOrPut(key);
        var user = entry.value_ptr;
        user.running = 0;
        user.pending = 0;
        user.n_partition = 0;
        return user;
    }

    pub fn parse_queue_entry(self: *Queue, line: []const u8) !void {
        var it = std.mem.tokenizeAny(u8, line, " ");
        var queue_items: [n_entry][]const u8 = undefined;
        var i: usize = 0;
        while (it.next()) |item| : (i += 1) {
            queue_items[i] = item;
        }
        if (i != 4) return error.BadNumberOfEntries;
        var user = try self.get_user(queue_items[0]);
        const running = std.mem.eql(u8, queue_items[1], "R");
        if (running) {
            user.running += 1;
            self.total_jobs += 1;
        } else {
            const n_pending = try parse_job_id(queue_items[3]);
            user.pending += n_pending;
            self.total_jobs += n_pending;
        }
        var part_iter = std.mem.tokenizeScalar(u8, queue_items[2], ',');
        while (part_iter.next()) |item| {
            const partition = std.mem.trim(u8, item, " ");
            const ip = try self.partition_id(partition);
            user.add_partition(ip);
        }
    }

    pub fn sorted_users(self: *const Queue) ![][]const u8 {
        var keys = try self.allocator.alloc([]const u8, self.users.count());
        var i: usize = 0;
        var it = self.users.keyIterator();
        while (it.next()) |item| : (i += 1) {
            keys[i] = item.*;
        }
        std.mem.sort([]const u8, keys, self.users, user_less_than);
        return keys;
    }
};

pub fn parse_job_id(job_id: []const u8) !usize {
    var i: usize = 0;
    while (i < job_id.len and job_id[i] != '[') : (i += 1) {}
    if (i == job_id.len) {
        return 1;
    }
    const jid = job_id[i + 1 .. job_id.len - 1];
    var it = std.mem.tokenizeAny(u8, jid, ",");
    var njob: usize = 0;
    while (it.next()) |elem| {
        i = 0;
        while (i < elem.len and elem[i] != '-') : (i += 1) {}
        if (i == elem.len) {
            njob += 1;
            continue;
        }
        const start_id = try std.fmt.parseInt(usize, elem[0..i], 10);
        const start = i + 1;
        i = start;
        while (i < elem.len and elem[i] >= '0' and elem[i] <= '9') : (i += 1) {}
        const end_id = try std.fmt.parseInt(usize, elem[start..i], 10);
        njob += end_id - start_id + 1;
    }
    return njob;
}

pub fn user_less_than(users: std.StringHashMap(User), a: []const u8, b: []const u8) bool {
    const user_a = users.get(a).?;
    const user_b = users.get(b).?;
    return (user_a.running + user_a.pending) > (user_b.running + user_b.pending);
}

pub fn main(init: std.process.Init) !void {
    var arena = std.heap.ArenaAllocator.init(std.heap.page_allocator);
    const allocator = arena.allocator();

    var queue = Queue.init(allocator);
    defer queue.deinit();

    var args: std.ArrayList([]const u8) = .empty;
    defer args.deinit(allocator);
    try args.appendSlice(allocator, &[_][]const u8{ "squeue", "--noheader", "-o %.20u %t %P %i" });
    var msg_end: []const u8 = undefined;

    // Parse arguments
    const argv = try init.minimal.args.toSlice(init.arena.allocator());
    if (argv.len == 2) {
        try args.append(allocator, try std.fmt.allocPrint(allocator, "--partition={s}", .{argv[1]}));
        msg_end = try std.fmt.allocPrint(allocator, "partition {s}", .{argv[1]});
    } else {
        msg_end = "the queue";
    }

    var proc = try std.process.spawn(init.io, .{ .argv = args.items, .stdout = .pipe });
    var buf: [4098]u8 = undefined;
    var reader = proc.stdout.?.reader(init.io, &buf);

    while (try reader.interface.takeDelimiter('\n')) |line| {
        const trimmed = std.mem.trimStart(u8, line, " ");
        try queue.parse_queue_entry(trimmed);
    }

    _ = try proc.wait(init.io);

    var writer = std.Io.File.stdout().writer(init.io, &buf);

    if (queue.total_jobs == 0) {
        try writer.interface.print("🥳🎉 There are no jobs in {s} 🎉🥳\n", .{msg_end});
        try writer.flush();
        return;
    }
    try writer.interface.print("There are {s}{}{s} jobs in {s}\n", .{ bold, queue.total_jobs, reset, msg_end });
    const user_names = try queue.sorted_users();

    for (user_names) |user_name| {
        const user = try queue.get_user(user_name);
        try user.print(&writer.interface, user_name, queue.partitions.items);
    }
}
