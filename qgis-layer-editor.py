cities_layer = QgsProject.instance().mapLayersByName('Merged')[0]
counties_layer = QgsProject.instance().mapLayersByName('REFER_COUNTY')[0]

if not cities_layer or not counties_layer:
    print("Please check the layer names and ensure they are loaded in the project.")
else:
    crs = cities_layer.crs().toWkt()
    output_layer = QgsVectorLayer(f'MultiPolygon?crs={crs}', 'municipals', 'memory')
    provider = output_layer.dataProvider()

    provider.addAttributes([
        QgsField('name', QVariant.String),
        QgsField('fips', QVariant.String),
        QgsField('county', QVariant.String)
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
                    new_feature['county'] = city_feature['COUNTY_CD']
                    if city_feature['CORPORATIO']:
                        new_feature['name'] = city_feature['CORPORATIO'] + " (City)"
                        new_feature['fips'] = city_feature['FIPS_CITY_']
                    else:
                        if city_feature['TOWNSHIP_N'] == 'URBAN':
                            continue
                        new_feature['name'] = city_feature['TOWNSHIP_N'] + " (Township)"
                        new_feature['fips'] = city_feature['FIPS_CODE']

                    provider.addFeature(new_feature)
                    break
    
    output_layer.commitChanges()
    
    QgsProject.instance().addMapLayer(output_layer)
    print("New layer created with municipal and county information.")