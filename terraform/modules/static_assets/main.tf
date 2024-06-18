// https://storage.googleapis.com/ts.krikis.online/index.html

locals {
  # A simple map to look up MIME types based on file extensions
  mime_types = {
    "css"  = "text/css"
    "html" = "text/html"
    "jpeg" = "image/jpeg"
    "jpg"  = "image/jpeg"
    "js"   = "application/javascript"
    "json" = "application/json"
    "ktx2" = "image/ktx2"
    "png"  = "image/png"
    "svg"  = "image/svg+xml"
    "ts" = "application/typescript"
    "wasm" = "application/wasm"
    "wgsl" = "application/webgpu"
  }
}

resource "google_storage_bucket" "static_assets" {
  name     = "ts.krikis.online"
  location = "EUROPE-CENTRAL2"
  uniform_bucket_level_access = true
  force_destroy = true

  // Later: This is not working, and I'm not quite sure why...
  website {
    main_page_suffix = "index.html"
    not_found_page   = "404.html"
  }

  cors {
    origin = ["*"]
    method = ["GET"]
    response_header = ["Content-Type"]
  }
}

resource "google_storage_bucket_iam_member" "public_access" {
  for_each = toset([
    "roles/storage.objectViewer"
  ])
  bucket   = google_storage_bucket.static_assets.name
  role     = each.key
  member   = "allUsers"
}

// Note - It will remove files if they get removed from the source directory, which is good!
resource "google_storage_bucket_object" "upload_files" {
  for_each = fileset("${path.module}/../../../static", "**")
  name     = each.value
  bucket   = google_storage_bucket.static_assets.name
  source   = join("/", ["${path.module}/../../../static", each.value])
  content_type = lookup(local.mime_types, regex(".*\\.([a-zA-Z0-9]+)$", each.value)[0], "application/octet-stream")
  cache_control = "public, max-age=60" # Later: Increase for the production setup.

  depends_on = [null_resource.build]
}

# Later: Improve this to actually build the static assets and possibly also the Docker containers!
resource "null_resource" "build" {
  triggers = {
    always_run = timestamp()
  }
}