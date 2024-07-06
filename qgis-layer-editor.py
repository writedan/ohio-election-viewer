cities_layer = QgsProject.instance().mapLayersByName('Merged')[0]
counties_layer = QgsProject.instance().mapLayersByName('REFER_COUNTY')[0]

if not cities_layer or not counties_layer:
    print("Please check the layer names and ensure they are loaded in the project.")
else:
    crs = cities_layer.crs().toWkt()
    output_layer = QgsVectorLayer(f'MultiPolygon?crs={crs}', 'city_with_county_info', 'memory')
    provider = output_layer.dataProvider()

    provider.addAttributes([
        QgsField('municipal_name', QVariant.String),
        QgsField('municipal_fips', QVariant.String),
        QgsField('county_name', QVariant.String),
        QgsField('canonical_county', QVariant.String),
        QgsField('county_fips', QVariant.String)
    ])
    output_layer.updateFields()

    spatial_index = QgsSpatialIndex()
    for county_feature in counties_layer.getFeatures():
        spatial_index.insertFeature(county_feature)

    for city_feature in cities_layer.getFeatures():
        city_point = city_feature.geometry()
        nearest_ids = spatial_index.intersects(city_point.boundingBox())
        
        if nearest_ids:
            for county_id in nearest_ids:
                county_feature = counties_layer.getFeature(county_id)
                county_polygon = county_feature.geometry()
                if city_point.intersects(county_polygon):
                    new_feature = QgsFeature()
                    new_feature.setGeometry(city_point)
                    new_feature.setFields(output_layer.fields())
                    new_feature['county_fips'] = ""
                    new_feature['county_name'] = ""
                    new_feature['canonical_county'] = city_feature['COUNTY_CD']
                    if city_feature['CORPORATIO']:
                        new_feature['municipal_name'] = city_feature['CORPORATIO'] + " (City)"
                        new_feature['municipal_fips'] = city_feature['FIPS_CITY_']
                    else:
                        if city_feature['TOWNSHIP_N'] == 'URBAN':
                            continue
                        new_feature['municipal_name'] = city_feature['TOWNSHIP_N'] + " (Township)"
                        new_feature['municipal_fips'] = city_feature['FIPS_CODE']

                    provider.addFeature(new_feature)
                    break
                    
    output_layer.commitChanges()
    output_layer.startEditing()
    
    for city_feature in output_layer.getFeatures():
        city_point = city_feature.geometry()
        for county_feature in counties_layer.getFeatures():
            county_polygon = county_feature.geometry()
            if city_point.intersects(county_polygon):
                county_name = county_feature['COUNTY']
                county_fips = county_feature['FIPS_COUNT']
                city_feature['county_fips'] += "," + county_fips
                city_feature['county_name'] += "," + county_name
                output_layer.updateFeature(city_feature)
    
    output_layer.commitChanges()
    
    QgsProject.instance().addMapLayer(output_layer)
    print("New layer created with municipal and county information.")