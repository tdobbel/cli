#!/usr/bin/env python

import argparse
from typing import List
import os
from osgeo import ogr


driver = ogr.GetDriverByName("ESRI Shapefile")
WORLD_POLYGONS = (
    "/home/dobbelaeret/Documents/osm/land-polygons-split-4326/land_polygons.shp"
)


def get_polygons_from_bbox(bounds: List[float], src_shp: str, dst_shp: str) -> None:
    minx, maxx, miny, maxy = bounds
    driver = ogr.GetDriverByName("ESRI Shapefile")
    src = driver.Open(src_shp, 0)
    src_lyr = src.GetLayer()
    if os.path.isfile(dst_shp):
        driver.DeleteDataSource(dst_shp)
    dst = driver.CreateDataSource(dst_shp)
    dst_lyr = dst.CreateLayer(
        "poly", src_lyr.GetSpatialRef(), geom_type=src_lyr.GetLayerDefn().GetGeomType()
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


def dissolve_polygons(src_shp: str, dst_shp: str) -> None:
    src = driver.Open(src_shp, 0)
    src_lyr = src.GetLayer()
    if os.path.isfile(dst_shp):
        driver.DeleteDataSource(dst_shp)
    dst = driver.CreateDataSource(dst_shp)
    dst_lyr = dst.CreateLayer(
        "dissolved",
        src_lyr.GetSpatialRef(),
        geom_type=src_lyr.GetLayerDefn().GetGeomType(),
    )
    lyr_def = dst_lyr.GetLayerDefn()
    multi = ogr.Geometry(ogr.wkbMultiPolygon)
    for src_feature in src_lyr:
        geom = src_feature.geometry()
        multi.AddGeometry(geom)
    union = multi.UnionCascaded()
    for geom in union:
        dst_feature = ogr.Feature(lyr_def)
        dst_feature.SetGeometry(geom)
        dst_lyr.CreateFeature(dst_feature)
        dst_feature = None
    src = dst = None


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument(
        "-b", "--bbox", nargs=4, type=float, help="Bounding box (minlon, maxlon, minlat, maxlat)"
    )
    parser.add_argument("-m", "--mesh", type=str, help="Mesh file")
    parser.add_argument("-o", "--output", type=str, required=True)
    args = parser.parse_args()
    has_mesh = args.mesh is not None
    has_bbox = args.bbox is not None
    if has_bbox == has_mesh:
        print("Bounding box or mesh file (not both) required for extraction")
        parser.print_help()
        return
    if has_bbox:
        minlon, maxlon, minlat, maxlat = args.bbox
        minlon -= 1
        maxlon += 1
        minlat -= 1
        maxlat += 1
    else:
        import slim4
        from slim4.tools import proj_transform

        mesh = slim4.d2.Mesh(args.mesh)
        lonlat = proj_transform(
            mesh.projection, "+proj=latlong +ellps=WGS84", mesh.xnodes
        )[:2]
        minlon, minlat = lonlat.min(axis=1) - 1
        maxlon, maxlat = lonlat.max(axis=1) + 1

    dst_shp = args.output
    if not dst_shp.endswith(".shp"):
        os.makedirs(dst_shp, exist_ok=True)
        dst_shp = os.path.join(dst_shp, dst_shp+".shp")

    get_polygons_from_bbox((minlon, maxlon, minlat, maxlat), WORLD_POLYGONS, "tmp.shp")
    dissolve_polygons("tmp.shp", dst_shp)
    driver.DeleteDataSource("tmp.shp")


if __name__ == "__main__":
    main()
