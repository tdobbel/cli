#!/usr/bin/env python

import argparse
import os
from osgeo import ogr

WORLD_REEFS = "/home/dobbelaeret/Documents/unep/14_001_WCMC008_CoralReefs2021_v4_1/01_Data/WCMC008_CoralReef2021_Py_v4_1.shp"

def get_polygons_from_bbox(bounds: list[float], src_shp: str, dst_shp: str) -> None:
    """Extract polygons from a shapefile that are within a bounding box."""
    driver = ogr.GetDriverByName('ESRI Shapefile')
    minx, maxx, miny, maxy = bounds
    src = driver.Open(src_shp,0)
    src_lyr = src.GetLayer()
    if os.path.isfile(dst_shp):
        driver.DeleteDataSource(dst_shp)
    dst = driver.CreateDataSource(dst_shp)
    dst_lyr = dst.CreateLayer(
        "reefs",
        src_lyr.GetSpatialRef(),
        geom_type=src_lyr.GetLayerDefn().GetGeomType()
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
    """Parse script arguments and extract reefs from shapefile."""
    parser = argparse.ArgumentParser()
    parser.add_argument("minlon", type=float, help="Minimum longitude")
    parser.add_argument("maxlon", type=float, help="Maximum longitude")
    parser.add_argument("minlat", type=float, help="Minimum latitude")
    parser.add_argument("maxlat", type=float, help="Maximum latitude")
    parser.add_argument("shp_file", type=str, help="Destination shapefile")
    args = parser.parse_args()
    buffer = 1.0
    bounds = [
        args.minlon-buffer, args.maxlon+buffer,
        args.minlat-buffer, args.maxlat+buffer
    ]
    get_polygons_from_bbox(bounds, WORLD_REEFS, args.shp_file)

if __name__ == "__main__":
    main()
