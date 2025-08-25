const std = @import("std");

const allocator = std.heap.c_allocator;
const ArrayList = std.array_list.Managed;
const HashMap = std.StringHashMap(ArrayList([]const u8));

pub fn main() !void {
    if (std.os.argv.len != 2) {
        std.debug.print("Missing input file\n", .{});
        return;
    }
    const cwd = std.fs.cwd();
    const file_name: [:0]const u8 = std.mem.span(std.os.argv[1]);
    const file = try cwd.openFile(file_name, .{});

    var biblio = HashMap.init(allocator);
    defer biblio.deinit();

    var buffer: [1024]u8 = undefined;
    var reader = file.reader(&buffer);

    var key: []const u8 = undefined;
    {
        defer file.close();
        while (reader.interface.takeDelimiterExclusive('\n')) |line| {
            if (line.len == 0) continue;
            if (line[0] == '@') {
                const istart = std.mem.indexOf(u8, line, "{").?;
                key = try allocator.dupe(u8, line[istart + 1 .. line.len - 1]);
            }
            const entry = try biblio.getOrPut(key);
            if (!entry.found_existing) {
                entry.value_ptr.* = ArrayList([]const u8).init(allocator);
            }
            const line_copy = try allocator.dupe(u8, line);
            errdefer allocator.free(line_copy);
            try entry.value_ptr.*.append(line_copy);
        } else |err| if (err != error.EndOfStream) {
            return err;
        }
    }
    var names: [][]const u8 = try allocator.alloc([]const u8, biblio.count());
    defer allocator.free(names);
    var it = biblio.keyIterator();
    var i: usize = 0;
    while (it.next()) |name| {
        names[i] = name.*;
        i += 1;
    }
    std.mem.sort([]const u8, names, {}, struct {
        pub fn lessThan(_: void, a: []const u8, b: []const u8) bool {
            return std.mem.order(u8, a, b) == .lt;
        }
    }.lessThan);

    const out_file = try cwd.createFile("sorted.bib", .{});
    defer out_file.close();
    var writer = out_file.writer(&buffer);
    for (names) |name| {
        const values = biblio.get(name).?;
        for (values.items) |line| {
            try writer.interface.print("{s}\n", .{line});
            allocator.free(line);
        }
        values.deinit();
        allocator.free(name);
    }
}
