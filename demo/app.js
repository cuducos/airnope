const spam = {
    segment: 'error',
    label: 'This message is spam :(',
    icon: 'thumbs down'
};
const not_spam = {
    segment: 'success',
    label: 'This message is not spam :)',
    icon: 'thumbs up'
};
const error = {
    segment: 'warning',
    label: 'An unexpected error occurred. Please try again later.',
    icon: 'warning'
};
const too_many_requests = {
    segment: 'warning',
    label: 'The server is too busy. Please try again later.',
    icon: 'warning'
}

function resultFor(config) {
    const msg = document.createElement('div');
    const icon = document.createElement('i');
    const label = document.createTextNode(config.label);

    msg.className = `ui ${config.segment} message`;
    msg.id = 'result'
    icon.className = `${config.icon} icon`;
    label.textContent = config.label;

    msg.appendChild(icon);
    msg.appendChild(label);

    return msg;
}

document.addEventListener('DOMContentLoaded', () => {
    const submit = document.getElementById('submit');
    const result = document.getElementById('result');
    const message = document.getElementById('message');

    submit.addEventListener('click', async () => {
        submit.disabled = true;
        if ( result.firstChild !== null ) result.removeChild(result.firstChild);
        try {
            const response = await fetch('https://airnope.onrender.com/', {
                method: 'POST',
                headers: {
                    'Accept': 'application/json',
                    'Content-Type': 'application/json'
                },
                body: JSON.stringify({ message: message.value }),
            });

            if (response.status === 429) {
                result.appendChild(resultFor(too_many_requests));
                submit.disabled = false;
                return;
            }
            if (!response.ok) {
                console.error(`HTTP Status ${response.status}: ${response.statusText}`);
                result.appendChild(resultFor(error));
                submit.disabled = false;
                return;
            }

            const data = await response.json();
            console.log(data);
            if (data.spam) {
                result.appendChild(resultFor(spam));
            } else {
                result.appendChild(resultFor(not_spam));
            }
            submit.disabled = false;
        } catch (e) {
            console.error(e);
            result.appendChild(resultFor(error));
            submit.disabled = false;
        }
    });
});

