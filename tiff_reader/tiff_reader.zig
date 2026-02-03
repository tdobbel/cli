const std = @import("std");
const builtin = @import("builtin");

const Endian = std.builtin.Endian;
const native_endian = builtin.cpu.arch.endian();

const TiffError = error{
    InvalidFirstBytes,
    BadMagicNumber,
};

const TiffReader = struct {
    data: []const u8,
    offset: usize,
    endian: Endian,

    pub fn new(data: []const u8) !TiffReader {
        if (std.mem.eql(u8, data[0..2], "II")) {
            return TiffReader{ .data = data, .offset = 0, .endian = .little };
        }
        if (std.mem.eql(u8, data[0..2], "MM")) {
            return TiffReader{ .data = data, .offset = 0, .endian = .big };
        }
        return TiffError.InvalidFirstBytes;
    }

    pub fn read_scalar(self: *TiffReader, comptime T: type) T {
        const shift = @sizeOf(T);
        const buffer: *const [@divExact(@typeInfo(T).int.bits, 8)]u8 = @ptrCast(self.data[self.offset .. self.offset + shift]);
        const value: T = @bitCast(buffer.*);
        self.offset += shift;
        return if (self.endian == native_endian) value else @byteSwap(value);
    }
};

pub fn main() !void {
    if (std.os.argv.len != 2) {
        return error.InvalidArgument;
    }
    const cwd = std.fs.cwd();
    const file_name: [:0]const u8 = std.mem.span(std.os.argv[1]);
    const file = try cwd.openFile(file_name, .{});
    const stat = try std.posix.fstat(file.handle);
    const map: []u8 = try std.posix.mmap(null, @intCast(stat.size), std.posix.PROT.READ, .{ .TYPE = .SHARED }, file.handle, 0);
    var tiff_reader: TiffReader = try TiffReader.new(map);
    tiff_reader.offset = 2;
    // std.debug.print("{}\n", .{tiff_reader.offset});
    const magic = tiff_reader.read_scalar(u16);
    std.debug.print("magic={}, offset={}\n", .{magic, tiff_reader.offset});
}
