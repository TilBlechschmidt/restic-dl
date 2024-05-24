fetch target resource:
	wget -O {{target}} {{resource}}

fetch-js:
	mkdir -p assets/js
	just fetch assets/js/htmx.js https://unpkg.com/htmx.org@1.9.12
	just fetch assets/js/htmx-sse.js https://unpkg.com/htmx.org@1.9.12/dist/ext/sse.js
	just fetch assets/js/clipboard.js https://unpkg.com/clipboard@2.0.11/dist/clipboard.min.js
