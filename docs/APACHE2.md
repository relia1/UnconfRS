### Apache2 Configuration

To deploy with Apache2 as a reverse proxy:

1. Copy the configuration file:
   ```bash
   sudo cp conf/apache2-unconfrs.conf /etc/apache2/sites-available/
   ```

2. Enable required modules:
   ```bash
   sudo a2enmod ssl proxy proxy_http headers rewrite
   ```

3. Update the configuration:
   - Replace `your-domain.com` with your actual domain name
   - Update SSL certificate paths in the configuration file

4. Enable the site and reload Apache:
   ```bash
   sudo a2ensite apache2-unconfrs.conf
   sudo systemctl reload apache2
   ```
