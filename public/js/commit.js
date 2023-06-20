const commit = document.getElementById("commit")
const preview_button = document.getElementById("preview")
const commit_button = document.getElementById("submit")

const username = document.getElementById("username")
const message = document.getElementById("message")


fetch("raw").then(response=>response.text()).then(text=>{
    commit.value = text
})
preview_button.onclick = () =>{
    fetch("commit", {
    method: 'POST',
    headers: {
        'Content-Type': 'text/plain'
    },
    body: commit.value})
   .then(response => response.text()).then(response => {
        try{
            JSON.parse(response)
        }catch(_){
            throw new Error('failure to parse response');
        }
        localStorage.setItem("data", response)
        let url = new URL(window.location.href)
        url = url.origin
        url +="?modified=true"
        window.open(url, "_blank")
    }).catch(e=>alert("error:", e))
}
commit_button.onclick = () =>{
    const m = {
        message: message.value,
        username: username.value,
        document: commit.value
    }
    fetch("submit", {
        method: 'POST',
        headers: {
            'Content-Type': 'application/json'
        },
        body: JSON.stringify(m)})
       .then(response => response.text()).then(text => alert(text))
}