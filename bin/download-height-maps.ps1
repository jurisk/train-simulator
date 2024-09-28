$ProgressPreference = 'SilentlyContinue' # It is very slow otherwise
Invoke-WebRequest https://storage.googleapis.com/ts-height-maps/usa/usa_east_etopo_2022.tiff -OutFile misc/assets-original/height-maps/usa/usa_east_etopo_2022.tiff
Invoke-WebRequest https://storage.googleapis.com/ts-height-maps/europe/europe_etopo_2022.tiff -OutFile misc/assets-original/height-maps/europe/europe_etopo_2022.tiff