const commits_container = document.getElementById("commits")
const modal = document.getElementById("myModal")
const modal_text = document.getElementById("modalText")
const preview = document.getElementById("preview")
const approve = document.getElementById("approve"), cancel = document.getElementById("cancel"), reject = document.getElementById("reject")

const getCookie = function (name) {
    var match = document.cookie.match(new RegExp('(^| )' + name + '=([^;]+)'));
    if (match) return match[2];
}

const setCookie = function (name, value) {
    document.cookie = name + "=" + (value || "") + "; path=/";
}

let pass = getCookie("pass")
if (!pass) {
    pass = prompt("Enter Password")
    setCookie("pass", pass)
}

let current_document;
let current_path;

fetch("api/commits", {
    headers: {
        'Authorization': `Bearer ${pass}`
    }
}).then(response => response.json()).then(entries => {
    for (const entry of entries) {
        const tr = document.createElement("tr")
        const td = document.createElement("td")
        td.textContent = entry.post.message
        const td1 = document.createElement("td")
        td1.textContent = entry.post.username
        tr.appendChild(td1)
        tr.appendChild(td)
        tr.onclick = () => {
            current_document = entry.post.document
            current_path = entry.path
            modal_text.textContent = entry.patch
            modal.showModal()
        }
        commits_container.appendChild(tr)
    }
}).catch(_=>{
    document.cookie = 'pass=; Max-Age=0'
    alert("incorrect pass")
})



cancel.onclick = () => {
    modal.close()
}
approve.onclick = () => {
    modal.close()
    const m = {
        path: current_path,
        kind: "Approve"
    }
    fetch("manage", {
        method: 'POST',
        headers: {
            'Content-Type': 'application/json',
            'Authorization': `Bearer ${pass}`
        },
        body: JSON.stringify(m)
    })
        .then(response => response.text()).then((text) => {
            const url = new URL(text)
            window.open(url, "_self")
        }).catch(_=>alert("unable to approve, try again later"))
}
reject.onclick = () => {
    modal.close()
    const m = {
        path: current_path,
        kind: "Reject"
    }
    fetch("manage", {
        method: 'POST',
        headers: {
            'Content-Type': 'application/json',
            'Authorization': `Bearer ${pass}`
        },
        body: JSON.stringify(m)
    })
        .then(response => response.text()).then(text => alert(text)).then(() => location.reload()).catch(_=>alert("unable to approve, try again later"))
}

preview.onclick = () => {
    fetch("commit", {
        method: 'POST',
        headers: {
            'Content-Type': 'text/plain',
        },
        body: current_document
    })
        .then(response => response.text()).then(response => {
            try {
                JSON.parse(response)
            } catch (_) {
                throw new Error('failure to parse response');
            }
            localStorage.setItem("data", response)
            let url = new URL(window.location.href)
            url = url.origin
            url += "?modified=true"
            window.open(url, "_blank")
        }).catch(e => alert("error:", e))
}