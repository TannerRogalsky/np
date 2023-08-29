terraform {
  required_providers {
    aws = {
      source  = "hashicorp/aws"
      version = "~> 5"
    }
  }

  backend "s3" {}
}

provider "aws" {
  region = "us-east-1"
  default_tags {
    tags = {
      project        = "tools-dev-machines",
      managed_by     = "terraform",
      group          = "tools",
      department     = "product"
      sub_department = "hardware-tools"
    }
  }
}

locals {
  db_password = "d3vm4ch1"
}
