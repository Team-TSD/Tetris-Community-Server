const content_box = document.getElementById("content")
const navBox = document.getElementById("nav")
const navLeft = document.getElementById("navLeft")
const navButton = document.getElementById("nav-button")
const urlParams = new URLSearchParams(window.location.search);
const contributor_box = document.getElementById("contributors")

navButton.onclick = () =>{
    if(navLeft.classList.contains("open")){
        navLeft.classList.remove("open")
        navButton.classList.remove("open")
    }else{
        navLeft.className+=" open"
        navButton.className+=" open"
    }
}

window.addEventListener('click', function(e){   
    if (!navLeft.contains(e.target) && !navButton.contains(e.target)){
        if(navLeft.classList.contains("open")){
            navLeft.classList.remove("open")
            navButton.classList.remove("open")
        }
    }
  });

const parsePages = function(pages){
    content_box.innerHTML = pages[0].text //introduction page
    let links = []
    for(const i in pages){
        const page = pages[i]
        const li = document.createElement("li")
        const a = document.createElement("a")
        a.classList = "nav-h1 nav-link"
        if(i == 0){
            a.classList += " active"
        }
        a.textContent = page.title
        a.addEventListener("click", ()=>{
            const current = document.getElementsByClassName("active")[0];
            if(current.nextSibling){
                current.nextSibling.style.display = "none"
            }
            current.classList.remove("active")
            content_box.innerHTML = page.text
            a.className+=" active"
            if(page.children.length > 0){
                a.nextSibling.style.display = "block"
            }
            window.scrollTo({top:0});
        })
        li.appendChild(a)
        links.push(a)
        if(page.children){
            const ul = document.createElement("ul")
            ul.classList = "nav-list-h2"
            ul.style.display = "none"
            for(const i in page.children){
                const p = page.children[i]
                const li = document.createElement("li")
                const a = document.createElement("a")
                a.href = `#${i}`
                a.classList = "nav-h2 nav-link"
                a.textContent = p
                ul.appendChild(a)
            }
            li.appendChild(ul)
        }
        navBox.appendChild(li)
    }
}

if(urlParams.get("modified")){
    const data = JSON.parse(localStorage.getItem("data"));
    parsePages(data)
}else{
    fetch("data").then(response => response.json()).then(pages=>parsePages(pages))
}

fetch("contributors").then(response =>response.json()).then(contributors =>{
    for(const i in contributors){
        if(i==contributors.length - 1){
            contributor_box.textContent += `${contributors[i]}`
        }else if(i==contributors.length - 2){
            contributor_box.textContent += `${contributors[i]}, and `
        }else{
            contributor_box.textContent += `${contributors[i]}, `
        }
    }
})
