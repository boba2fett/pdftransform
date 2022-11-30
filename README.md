# pdftransform

- Using Pdfium to translate source documents into output documents
- look into api.http
- to run in dev, get pdfium, start mongodb, provide MONGO_URI (defaults to "mongodb://localhost:27017")

## Runtime environment variables

- EXPIRE_AFTER_SECONDS=90000 25h expire time for jobs (always set the same with same mongodb as index of mongodb is not updated)
- PARALLELISM=10 parallel downloads for source files
- MAX_KIBIBYTES=4048 maximum for preview input files

## TODOs

## Done

- transform /tmp usage to database to allow running in cluster (done for result files)
- validate rotation only for single pages (done by turning all required pages)
- make environment variable for mongo_uri nicer (rewrite env variable)
- validate jobs for doubled source id's (no race between files) or destination id's (destination is partly fixed by using mongo gridfs)
- pin dependencies (pdfium, docker container) (for ease of use won't fix)
- use better mechanism for setting _links (moved building routes closer to actual routes)
- group logs for individual jobs (using differnt logger that allows setting attributes)
- deleting old jobs (using expire_after with default 25h and for gridfs setting uploadDate on chunks collection)
- cache open sourcefiles (if in order)
- providing page count or preview images
- readd MimeTypes
- Added /health route
- evaluate cancel of jobs (time is to precious)
- retry failed operations like file download or callback (wont't fix)
- further design desicions regarding running in cluster (wont't fix)
- test preview exensivley (wont't fix)
