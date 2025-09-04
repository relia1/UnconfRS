document.addEventListener('DOMContentLoaded', function() {
    // Handle logout functionality
    const logout = document.getElementById('logout');
    if (logout) {
        document.getElementById('logout').addEventListener('click', async function(event) {
            event.preventDefault();
            try {
                const response = await fetch('/api/v1/logout', {
                    method: 'POST',
                });

                if (response.ok) {
                    location.href = '/login';
                } else {
                    console.error('Logout failed with status: ', response.status);
                    alert('Failed to logout');
                }
            } catch (error) {
                console.error(error);
                alert('Failed to logout');
            }
        });
    }

    // Handle active navigation highlighting
    const currentPath = window.location.pathname;
    const navLinks    = document.querySelectorAll('.nav-link');

    // Remove active class from all links first
    navLinks.forEach(link => {
        link.classList.remove('active');
    });

    // Add active class to the current page link
    navLinks.forEach(link => {
        const href = link.getAttribute('href');
        if (href && (
            (currentPath === '/' && href === '/') ||
            (currentPath !== '/' && href !== '/' && currentPath.startsWith(href))
        )) {
            link.classList.add('active');
        }
    });
});
