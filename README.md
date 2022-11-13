# pdftransform

- Using Pdfium to translate source documents into output documents
- post content from example.json to /convert
- to run in dev, get pdfium, start mongodb, provide MONGO_URI (defaults to "mongodb://localhost:27017")

## TODOs

- deleting old jobs
- retry failed operations like file download or callback

## Done

- transform /tmp usage to database to allow running in cluster (done for result files)
- validate rotation only for single pages (done by turning all required pages)
- make environment variable for mongo_uri nicer (rewrite env variable)
- validate jobs for doubled source id's (no race between files) or destination id's (destination is partly fixed by using mongo gridfs)
- pin dependencies (pdfium, docker container) (for ease of use won't fix)
- use better mechanism for setting _links (moved building routes closer to actual routes)
- group logs for individual jobs (using differnt logger that allows setting attributes)
