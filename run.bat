@echo off
echo Starting MiniRust using Docker Compose...
docker compose up -d --build
echo.
echo The project is running in the background!
echo Web UI: http://localhost:3001
echo API:    http://localhost:3000
echo.
echo (You can view logs using: docker compose logs -f)
pause
