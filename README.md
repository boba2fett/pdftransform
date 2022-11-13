# pdftransform

- Using Pdfium to translate source documents into output documents
- post content from example.json to /convert
- to run in dev, get pdfium, start mongodb, provide MONGO_URI (defaults to "mongodb://localhost:27017")

## TODOs

- group logs for individual jobs
- use better mechanism for setting _links
- deleting old jobs
- retry failed operations like file download or callback
- pin dependencies (pdfium, docker container)

## Done

- transform /tmp usage to database to allow running in cluster (done for result files)
- validate rotation only for single pages (done by turning all required pages)
- make environment variable for mongo_uri nicer (rewrite env variable)
- validate jobs for doubled source id's (no race between files) or destination id's (destination is partly fixed by using mongo gridfs)
