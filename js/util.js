
export function numberWithCommas(x) {
    return x.toString().replace(/\B(?=(\d{3})+(?!\d))/g, ",");
}

export function removeFileExtension(x) {
    return x.replace(/\.[^/.]+$/, "");
}
