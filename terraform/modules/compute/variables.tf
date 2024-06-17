variable "container_image" {
  description = "The container image to deploy"
  type        = string
}

variable "project_id" {
  description = "The project ID to deploy resources into"
  type        = string
}

variable "network_name" {
  description = "The name of the network to deploy instances into"
  type        = string
}

variable "instance_name" {
  description = "The desired name to assign to the deployed instance"
  type        = string
}

variable "zone" {
  description = "The GCP zone to deploy instances into"
  type        = string
}

variable "client_email" {
  description = "Service account email address"
  type        = string
  default     = ""
}

variable "cos_image_name" {
  description = "The forced COS image to use instead of latest"
  default     = "cos-stable-77-12371-89-0"
}

variable "env_vars" {
  description = "A map of environment variables to set in the container"
  type        = map(string)
  default     = {}
}

variable "volumes" {
  type = map(object({
    name       = string
    host_path  = string
    mount_path = string
  }))

  default = {}
}