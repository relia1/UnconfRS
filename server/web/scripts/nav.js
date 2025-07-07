const mobileMenu = document.getElementById('mobile-menu');
const navLinks = document.querySelector('.nav-links');
mobileMenu.addEventListener('click', () => {
    navLinks.classList.toggle('nav-active');
});

document.addEventListener('DOMContentLoaded', function() {
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
});
