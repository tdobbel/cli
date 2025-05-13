#!/usr/bin/env python

from typing import List
import argparse
import os
from osgeo import ogr


WORLD_COAST = "/home/dobbelaeret/Documents/osm/coastlines-split-4326/lines.shp"


def get_lines_from_bbox(bounds: List[float], src_shp: str, dst_shp: str) -> None:
    minx, maxx, miny, maxy = bounds
    driver = ogr.GetDriverByName("ESRI Shapefile")
    src = driver.Open(src_shp, 0)
    src_lyr = src.GetLayer()
    if os.path.isfile(dst_shp):
        driver.DeleteDataSource(dst_shp)
    dst = driver.CreateDataSource(dst_shp)
    dst_lyr = dst.CreateLayer(
        "coastlines",
        src_lyr.GetSpatialRef(),
        geom_type=src_lyr.GetLayerDefn().GetGeomType(),
    )
    lyr_def = dst_lyr.GetLayerDefn()
    for src_feature in src_lyr:
        geom = src_feature.geometry()
        xmin, xmax, ymin, ymax = geom.GetEnvelope()
        if xmin > maxx or ymin > maxy or xmax < minx or ymax < miny:
            continue
        dst_feature = ogr.Feature(lyr_def)
        dst_feature.SetGeometry(geom)
        dst_lyr.CreateFeature(dst_feature)
        dst_feature = None
    src = dst = None


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument(
        "-b",
        "--bbox",
        nargs=4,
        type=float,
        help="Bounding box (minlon, maxlon, minlat, maxlat)",
        required=True,
    )
    parser.add_argument("-o", "--output", type=str, required=True)
    args = parser.parse_args()
    dst_shp = args.output
    if not dst_shp.endswith(".shp"):
        os.makedirs(dst_shp, exist_ok=True)
        dst_shp = os.path.join(dst_shp, dst_shp + ".shp")
    get_lines_from_bbox(args.bbox, WORLD_COAST, dst_shp)


if __name__ == "__main__":
    main()
