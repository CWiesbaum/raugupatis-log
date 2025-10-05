#!/bin/bash
# Production deployment script

set -e

APP_NAME="raugupatis-log"
VERSION=${1:-latest}

echo "🚀 Deploying Raugupatis Log version: $VERSION"

# Build the Docker image
echo "Building Docker image..."
docker build -t $APP_NAME:$VERSION .

# Tag for production
docker tag $APP_NAME:$VERSION $APP_NAME:latest

echo "✅ Build complete!"
echo ""
echo "To run the container:"
echo "  docker run -p 3000:3000 -v \$(pwd)/data:/app/data $APP_NAME:$VERSION"
echo ""
echo "To run with Docker Compose:"
echo "  docker-compose up -d"
echo ""
echo "Health check endpoint: http://localhost:3000/health"