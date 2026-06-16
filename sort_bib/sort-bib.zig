const std = @import("std");

pub fn less_than_fn(_: void, a: []const u8, b: []const u8) bool {
    const n = @min(a.len, b.len);
    for (0..n) |i| {
        if (a[i] == b[i]) continue;
        return a[i] < b[i];
    }
    return false;
}

pub fn main(init: std.process.Init) !void {
    const alloc = init.arena.allocator();
    const args = try init.minimal.args.toSlice(alloc);
    if (args.len != 2) {
        return error.MissingInputFile;
    }
    const io = init.io;
    const contents = try std.Io.Dir.readFileAlloc(std.Io.Dir.cwd(), io, args[1], alloc, .unlimited);
    var split_iter = std.mem.splitScalar(u8, contents, '@');
    var citations = std.StringHashMap([]const u8).init(alloc);
    var label_array = std.array_list.Managed([]const u8).init(alloc);
    while (split_iter.next()) |item| {
        if (item.len == 0) continue;
        const start = std.mem.findPos(u8, item, 0, "{").?;
        const stop = std.mem.findPos(u8, item, start + 1, ",").?;
        const label = item[start + 1 .. stop];
        const key = try alloc.dupe(u8, label);
        for (0..key.len) |i| {
            key[i] = std.ascii.toLower(key[i]);
        }
        try citations.put(key, item);
        try label_array.append(key);
    }
    const labels = try label_array.toOwnedSlice();
    std.mem.sort([]const u8, labels, {}, less_than_fn);
    const out_file = try std.Io.Dir.cwd().createFile(io, "sorted.bib", .{});
    for (labels) |key| {
        const offset = try out_file.length(io);
        _ = try out_file.writePositionalAll(io, "@", offset);
        const content = citations.get(key).?;
        _ = try out_file.writePositionalAll(io, content, offset + 1);
    }
}
