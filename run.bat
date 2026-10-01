@echo off
echo Starting MiniRust using Docker Compose...
docker compose up -d --build
echo.
echo The project is running in the background!
echo.
echo All traffic goes through Caddy (reverse proxy):
echo   Web UI:  http://localhost
echo   API:     http://localhost/api/v1/...
echo.
echo (You can view logs using: docker compose logs -f)
pause
