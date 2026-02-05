const std = @import("std");
const builtin = @import("builtin");

const Endian = std.builtin.Endian;
const native_endian = builtin.cpu.arch.endian();

const TiffDataType = enum(u16) {
    short = 3,
    long = 4,
    float = 11,
    double = 12,
};

const TiffError = error{ InvalidFirstBytes, BadMagicNumber, InvalidDataType, UnsupportedBigTiff, UnknownTag, TooManyIFDs, InvalidTransformation };

const IfdEntry = struct {
    tag: u16,
    field_type: u16,
    count: u32,
    value_offset: u32,
};

const TiffIfd = struct {
    image_width: u32,
    image_length: u32,
    bits_per_sample: u16,
    compression: u16,
    photometric_interpretation: u16,
    samples_per_pixel: u16,
    strip_offsets: []u32,
    rows_per_strip: u32,
    planar_configuration: u16,
    sample_format: u16,
    strip_byte_counts: []u32,
    projection: []u8,
    model_tie_points: ?[]f64,
    model_pixel_scale_tag: ?[]f64,
    model_transformation_tag: ?[16]f64,
    geo_double_params_tag: ?[]f64,

    pub fn init(allocator: std.mem.Allocator) !*TiffIfd {
        var ifd = try allocator.create(TiffIfd);
        ifd.model_tie_points = null;
        ifd.model_pixel_scale_tag = null;
        ifd.model_transformation_tag = null;
        ifd.geo_double_params_tag = null;
        return ifd;
    }
};

const TiffDataset = struct {
    ifd: *TiffIfd,
    x: []f64,
    y: []f64,
    data: ?[]f32,

    pub fn from_ifd(allocator: std.mem.Allocator, ifd: *TiffIfd) !TiffDataset {
        const nx = ifd.image_width;
        const ny = ifd.image_length;
        var x = try allocator.alloc(f64, nx);
        var y = try allocator.alloc(f64, ny);
        const eps = std.math.floatEps(f64);
        if (ifd.model_transformation_tag) |trans| {
            if (@abs(trans[1]) > eps or @abs(trans[4]) > eps) {
                return TiffError.InvalidTransformation;
            }
            for (0..nx) |i| {
                x[i] = trans[3] + trans[0] * @as(f64, @floatFromInt(i));
            }
            for (0..ny) |i| {
                y[i] = trans[7] + trans[5] * @as(f64, @floatFromInt(i));
            }
            return TiffDataset{ .ifd = ifd, .x = x, .y = y, .data = null };
        }
        if (ifd.model_tie_points != null and ifd.model_pixel_scale_tag != null) {
            // Assume upper left corner is provided
            const pixel_scale = ifd.model_pixel_scale_tag.?;
            const tie_points = ifd.model_tie_points.?;
            if (tie_points.len != 6) {
                std.debug.print("model_tie_points has unexpected size", .{});
                return TiffError.InvalidTransformation;
            }
            const t0: u64 = @intFromFloat(tie_points[0]);
            const t1: u64 = @intFromFloat(tie_points[1]);
            if (t0 != 0 or t1 != 0) {
                std.debug.print("model_tie_point is not (0, 0)", .{});
                return TiffError.InvalidTransformation;
            }
            x[0] = tie_points[3];
            for (1..nx) |i| {
                x[i] = x[i - 1] + pixel_scale[0];
            }
            y[0] = tie_points[4];
            for (1..ny) |i| {
                y[i] = y[i - 1] - pixel_scale[1];
            }
            return TiffDataset{ .ifd = ifd, .x = x, .y = y, .data = null };
        }
        return TiffError.InvalidTransformation;
    }

    pub fn get_extent(self: *const TiffDataset) [4]f64 {
        const nx = self.ifd.image_width;
        const ny = self.ifd.image_length;
        var xmin = self.x[0];
        var xmax = self.x[nx - 1];
        if (xmin > xmax) {
            std.mem.swap(f64, &xmin, &xmax);
        }
        var ymax = self.y[0];
        var ymin = self.y[ny - 1];
        if (ymin > ymax) {
            std.mem.swap(f64, &ymin, &ymax);
        }
        return .{ xmin, xmax, ymin, ymax };
    }

    pub fn load_data(self: *TiffDataset, reader: *TiffReader) !void {
        var data = try reader.allocator.alloc(f32, self.ifd.image_length * self.ifd.image_width);
        var pixel: usize = 0;
        for (self.ifd.strip_offsets, 0..) |offset, strip| {
            reader.offset = @intCast(offset);
            const nmax = self.ifd.strip_byte_counts[strip] / 4;
            for (0..nmax) |_| {
                data[pixel] = reader.read_scalar(f32);
                pixel += 1;
            }
        }
        self.data = data;
    }
};

const TiffReader = struct {
    allocator: std.mem.Allocator,
    data: []const u8,
    offset: usize,
    endian: Endian,

    pub fn new(allocator: std.mem.Allocator, data: []const u8) !TiffReader {
        if (std.mem.eql(u8, data[0..2], "II")) {
            return TiffReader{ .allocator = allocator, .data = data, .offset = 2, .endian = .little };
        }
        if (std.mem.eql(u8, data[0..2], "MM")) {
            return TiffReader{ .allocator = allocator, .data = data, .offset = 2, .endian = .big };
        }
        return TiffError.InvalidFirstBytes;
    }

    pub fn read_scalar(self: *TiffReader, comptime T: type) T {
        const shift = @sizeOf(T);
        const buffer: *const [@sizeOf(T)]u8 = @ptrCast(self.data[self.offset .. self.offset + shift]);
        self.offset += shift;
        if (@typeInfo(T) == .float) {
            const value: @Type(.{ .int = .{ .signedness = .unsigned, .bits = @typeInfo(T).float.bits } }) = @bitCast(buffer.*);
            return if (self.endian == native_endian) @bitCast(value) else @bitCast(@byteSwap(value));
        }
        const value: T = @bitCast(buffer.*);
        return if (self.endian == native_endian) value else @byteSwap(value);
    }

    pub fn read_int_vector(self: *TiffReader, comptime T: type, entry: *const IfdEntry) ![]T {
        if (@typeInfo(T) != .int) {
            return TiffError.InvalidDataType;
        }
        var vec = try self.allocator.alloc(T, entry.count);
        const current = self.offset;
        self.offset = @intCast(entry.value_offset);
        for (0..entry.count) |i| {
            switch (entry.field_type) {
                @intFromEnum(TiffDataType.short) => {
                    vec[i] = @intCast(self.read_scalar(u16));
                },
                @intFromEnum(TiffDataType.long) => {
                    vec[i] = @intCast(self.read_scalar(u32));
                },
                else => {
                    return TiffError.InvalidDataType;
                },
            }
        }
        self.offset = current;
        return vec;
    }

    pub fn read_float_vector(self: *TiffReader, comptime T: type, entry: *const IfdEntry) ![]T {
        if (@typeInfo(T) != .float) {
            return TiffError.InvalidDataType;
        }
        var vec = try self.allocator.alloc(T, entry.count);
        const current = self.offset;
        self.offset = @intCast(entry.value_offset);
        for (0..entry.count) |i| {
            switch (entry.field_type) {
                @intFromEnum(TiffDataType.float) => {
                    vec[i] = @floatCast(self.read_scalar(f32));
                },
                @intFromEnum(TiffDataType.double) => {
                    vec[i] = @floatCast(self.read_scalar(f64));
                },
                else => {
                    return TiffError.InvalidDataType;
                },
            }
        }
        self.offset = current;
        return vec;
    }

    pub fn read_ifd_entry(self: *TiffReader) IfdEntry {
        const tag = self.read_scalar(u16);
        const field_type = self.read_scalar(u16);
        const count = self.read_scalar(u32);
        const value_offset = self.read_scalar(u32);
        return IfdEntry{ .tag = tag, .field_type = field_type, .count = count, .value_offset = value_offset };
    }

    pub fn process_ifd_entry(self: *TiffReader, ifd: *TiffIfd) !void {
        const entry = self.read_ifd_entry();
        switch (entry.tag) {
            256 => {
                ifd.image_width = entry.value_offset;
            },
            257 => {
                ifd.image_length = entry.value_offset;
            },
            258 => {
                ifd.bits_per_sample = @intCast(entry.value_offset);
            },
            259 => {
                ifd.compression = @intCast(entry.value_offset);
            },
            262 => {
                ifd.photometric_interpretation = @intCast(entry.value_offset);
            },
            273 => {
                ifd.strip_offsets = try self.read_int_vector(u32, &entry);
            },
            277 => {
                ifd.samples_per_pixel = @intCast(entry.value_offset);
            },
            278 => {
                ifd.rows_per_strip = entry.value_offset;
            },
            279 => {
                ifd.strip_byte_counts = try self.read_int_vector(u32, &entry);
            },
            284 => {
                ifd.planar_configuration = @intCast(entry.value_offset);
            },
            339 => {
                ifd.sample_format = @intCast(entry.value_offset);
            },
            33922 => {
                ifd.model_tie_points = try self.read_float_vector(f64, &entry);
            },
            33550 => {
                ifd.model_pixel_scale_tag = try self.read_float_vector(f64, &entry);
            },
            34264 => {
                const vec = try self.read_float_vector(f64, &entry);
                var trans: [16]f64 = undefined;
                @memcpy(&trans, vec);
                ifd.model_transformation_tag = trans;
                self.allocator.free(vec);
            },
            34735 => {},
            34736 => {
                ifd.geo_double_params_tag = try self.read_float_vector(f64, &entry);
            },
            34737 => {
                const start = entry.value_offset;
                const stop = start + entry.count;
                ifd.projection = try self.allocator.dupe(u8, self.data[start..stop]);
            },
            else => {
                return TiffError.UnknownTag;
            },
        }
    }

    pub fn read_tiff(self: *TiffReader) !TiffDataset {
        self.offset = 2;
        const magic = self.read_scalar(u16);
        if (magic == 43) {
            return TiffError.UnsupportedBigTiff;
        } else if (magic != 42) {
            std.debug.print("Big Tiff format currently not supported\n", .{});
            return TiffError.BadMagicNumber;
        }

        self.offset = @intCast(self.read_scalar(u32));
        const n_entry = self.read_scalar(u16);
        const ifd = try TiffIfd.init(self.allocator);
        for (0..n_entry) |_| {
            try self.process_ifd_entry(ifd);
        }
        if (self.read_scalar(u32) != 0) {
            std.debug.print("Found more than 1 IFD\n", .{});
            return TiffError.TooManyIFDs;
        }
        if (ifd.sample_format != 3) {
            std.debug.print("Only deals with float sample at the moment\n", .{});
            return TiffError.InvalidDataType;
        }
        return try TiffDataset.from_ifd(self.allocator, ifd);
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
    var arena: std.heap.ArenaAllocator = .init(std.heap.c_allocator);
    defer arena.deinit();
    const allocator = arena.allocator();
    const map: []u8 = try std.posix.mmap(null, @intCast(stat.size), std.posix.PROT.READ, .{ .TYPE = .SHARED }, file.handle, 0);
    var tiff_reader: TiffReader = try TiffReader.new(allocator, map);
    var tiff_data = try tiff_reader.read_tiff();
    std.debug.print("{any}\n", .{tiff_data.get_extent()});
    try tiff_data.load_data(&tiff_reader);
    const size = tiff_data.data.?.len;
    std.debug.print("data[{}]={}\n", .{ size - 1, tiff_data.data.?[size - 1] });
}
