const getFirstTabName = (tabList) => {
    const firstTab = tabList.querySelectorAll("[data-tab-name]")[0];
    firstTab.classList.add("active");
    firstTab.setAttribute("aria-selected", "true");
    firstTab.removeAttribute("tabIndex");

    return firstTab.dataset.tabName;
}

const openTab = (e) => {
    e.target.scrollIntoView({ behavior: "smooth", block: "nearest", inline: "center" });
    const currentTabContainer = e.target.closest('[role="tablist"]')
    currentTabContainer.querySelectorAll("[data-tab-content]").forEach((item) => {
        item.style.display = "none";
        item.setAttribute("role", "tabpanel");
    });

    currentTabContainer.querySelectorAll("[data-tab-name]").forEach((item) => {
        item.classList.remove("active");
        item.setAttribute("aria-selected", "false");
    });

    currentTabContainer.querySelector(
        `[data-tab-content="${e.target.dataset.tabName}"]`
    ).style.display = "block";

    const currentTab = currentTabContainer.querySelector(
        `[data-tab-name="${e.target.dataset.tabName}"]`
    )
    currentTab.style.display = "block";
    currentTab.classList.add("active");
    currentTab.setAttribute("aria-selected", "true");
    currentTab.removeAttribute("tabIndex");

    currentTabContainer.querySelectorAll('[role="tablist"]').forEach((tablist, tabSystemIdx) => {
        const firstTabName = getFirstTabName(tablist);

        if (tablist.querySelectorAll("[data-tab-content]").length) {
            tablist.querySelector(`[data-tab-content="${firstTabName}"]`).style.display = "block";
        }
    })
};

document.querySelectorAll('[role="tablist"]').forEach((tablist, tabSystemIdx) => {
    tablist.querySelectorAll("[data-tab-name]").forEach((item, idx) => {
        item.addEventListener("click", openTab);
        item.classList.add("select-tab");
        item.setAttribute("role", "tab");
        item.setAttribute("aria-selected", "false");
        item.setAttribute("aria-controls", `tab-system-${tabSystemIdx}-tab-${idx}`);

        item.id = `tab-${tabSystemIdx}-tab-${idx}`;
    });

    const firstTabName = getFirstTabName(tablist);

    if (tablist.querySelectorAll("[data-tab-content]").length) {
        tablist.querySelector(`[data-tab-content="${firstTabName}"]`).style.display = "block";
        tablist.querySelectorAll("[data-tab-content]").forEach((tabContent, idx) => {
            tabContent.setAttribute("role", "tabpanel");
            tabContent.setAttribute("tabindex", "0");
            tabContent.setAttribute("aria-labelledby", `tab-${tabSystemIdx}-tab-${idx}`);
            tabContent.setAttribute("id", `tab-system-${tabSystemIdx}-tab-${idx}`);
        });
    }
});