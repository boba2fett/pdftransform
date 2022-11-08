# pdftransform

- Using Pdfium to translate source documents into output documents
- post content from example.json to /convert
- to run in dev, get pdfium, start mongodb, provide ROCKET_DATABASES='{db={url="mongodb://localhost:27017"}}'

## TODOs

- group logs for individual jobs
- use better mechanism for setting _links
- transform /tmp usage to database to allow running in cluster (done for result files)
- deleting old jobs
- validate jobs for doubled source id's or destination id's
- validate rotation only for single pages
- retry failed operations like file download or callback
- pin dependencies
