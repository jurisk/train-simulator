# Inspired from https://github.com/terraform-google-modules/terraform-google-container-vm/tree/master/examples/simple_instance

locals {
  instance_name = format("%s-%s", var.instance_name, substr(md5(module.gce-container.container.image), 0, 8))
}

module "gce-container" {
  source  = "terraform-google-modules/container-vm/google"
  version = "~> 3.0"

  cos_image_name = var.cos_image_name

  container = {
    image = var.container_image

    # Could not launch threads otherwise
    securityContext = {
      privileged: true
    }

    env = [
      for key, value in var.env_vars : {
        name  = key
        value = value
      }
    ]
  }

  restart_policy = "Never" # Always, OnFailure, UnlessStopped are the other options
}

resource "google_compute_instance" "vm" {
  project      = var.project_id
  name         = local.instance_name
  machine_type = "e2-micro"
  zone         = var.zone

  boot_disk {
    initialize_params {
      image = module.gce-container.source_image
    }
  }

  network_interface {
    network = var.network_name
    access_config {}
  }

  tags = []

  metadata = {
    gce-container-declaration = module.gce-container.metadata_value
    google-logging-enabled    = "true"
    google-monitoring-enabled = "true"
  }

  labels = {
    container-vm = module.gce-container.vm_container_label
  }

  service_account {
    email = var.client_email
    scopes = [
      "https://www.googleapis.com/auth/cloud-platform",
    ]
  }
}
