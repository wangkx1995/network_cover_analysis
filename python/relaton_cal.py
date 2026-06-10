# %%
import pandas as pd
import geopandas as gpd
from shapely import wkt

# %%
poi_message = 'anhui_202507_ztest/message.csv'
poi_dtype_dict = {
    'jt_poi_id': 'string',
}
poi_df = pd.read_csv(poi_message, dtype=poi_dtype_dict).astype('string')
# %%
poi_df['geometry'] = poi_df['polygon_geom'].apply(lambda x: wkt.loads(x))

# 创建GeoDataFrame，设置坐标系为4326
gdf_poi = gpd.GeoDataFrame(poi_df, geometry='geometry', crs='EPSG:4326')
# %%
micro = gdf_poi[gdf_poi['data_type'] == '微网格']
market = gdf_poi[gdf_poi['data_type'] == '市场网格']
print(micro.shape)
print(market.shape)
# %%
# 先投影到EPSG:3857（单位为米），再进行空间操作和面积计算
micro_projected = micro.to_crs('EPSG:3857')
market_projected = market.to_crs('EPSG:3857')

micro_projected.sindex
market_projected.sindex
# %%
# sjoin只获取关联关系，会丢弃右表的geometry列
result = gpd.sjoin(
    micro_projected,
    market_projected,
    how='left',
    predicate='intersects'
)
# 保留sjoin左表索引，作为微网格唯一行标识
result['micro_row_id'] = result.index

print(result.shape)
# %%
# 通过index_right将右表(market)的geometry合并回来
market_geom_df = market_projected[['geometry']].rename(columns={'geometry': 'geometry_market'})
result = result.merge(market_geom_df, left_on='index_right', right_index=True, how='left')

# 重命名左表的active geometry列
result = result.rename_geometry('geometry_micro')


# %%
# 计算相交面积（几何体已经是投影坐标系，单位：平方米）
def calculate_intersection_area(row):
    try:
        geom_micro = row.get('geometry_micro')
        geom_market = row.get('geometry_market')
        if geom_micro is not None and geom_market is not None:
            intersection = geom_micro.intersection(geom_market)
            return intersection.area
        else:
            return 0.0
    except:
        return 0.0


result['intersection_area'] = result.apply(calculate_intersection_area, axis=1)
# 计算左表(微网格)面积
result['micro_area'] = result['geometry_micro'].apply(lambda x: x.area if x is not None else 0.0)

# 计算相交面积占左表面积占比
result['intersection_ratio'] = result['intersection_area'] / result['micro_area']

print(f"原始关联行数: {result.shape[0]}")
# %%
# 每个微网格取面积占比最大的一行
result_max = (
    result
    .sort_values(['micro_row_id', 'intersection_ratio', 'intersection_area'], ascending=[True, False, False])
    .drop_duplicates(subset=['micro_row_id'], keep='first')
)
# %%
# 保存结果到CSV
result_output = result_max.copy()
# 将几何对象转换为WKT字符串以便保存
if 'geometry_micro' in result_output.columns:
    result_output['geometry_micro_wkt'] = result_output['geometry_micro'].apply(lambda x: x.wkt if x else None)
if 'geometry_market' in result_output.columns:
    result_output['geometry_market_wkt'] = result_output['geometry_market'].apply(lambda x: x.wkt if x else None)

# 删除几何列，只保留WKT字符串
result_output = result_output.drop(columns=['geometry_micro', 'geometry_market'], errors='ignore')
