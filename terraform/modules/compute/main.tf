resource "google_compute_instance_template" "train_simulator_template" {
  name        = "train-simulator-template"
  machine_type = "e2-micro"

  tags = ["game-server"]

  disk {
    source_image = "projects/cos-cloud/global/images/family/cos-stable" # Container-Optimized OS
    auto_delete  = true
    boot         = true
  }

  network_interface {
    network = var.network_name
    access_config {
      # Ephemeral public IP
    }
  }

  # TODO: Use var.container_port for the containerPort instead of hardcoding it
  metadata = {
    "gce-container-declaration" = <<-EOF
      spec:
        containers:
          - name: train-simulator
            image: gcr.io/train-simulator-gcp/train-simulator
            ports:
              - name: http
                containerPort: 8080
        restartPolicy: Always
    EOF
  }
}

resource "google_compute_instance_from_template" "train_simulator_instance" {
  name       = "train-simulator-instance"
  zone       = var.gcp_zone
  source_instance_template = google_compute_instance_template.train_simulator_template.id
}