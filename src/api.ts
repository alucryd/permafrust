import axios from "axios";

const axiosAPI = axios.create({
    baseURL: "http://localhost:8080/api"
});

const axiosRequest = (method, url, request) => {
    const headers = {
        authorization: ""
    };
    return axiosAPI({
        method,
        url,
        data: request,
        headers
    }).then(res => {
        return Promise.resolve(res.data);
    })
        .catch(err => {
            return Promise.reject(err);
        });
};

const getRequest = (url, request) => axiosRequest("get", url, request);
const postRequest = (url, request) => axiosRequest("post", url, request);
const putRequest = (url, request) => axiosRequest("put", url, request);
const deleteRequest = (url, request) => axiosRequest("delete", url, request);

const listRootDirectories = async () => {
    try {
        const response = await getRequest("/root-directories", null);
        return response;
    } catch (error) {
        console.error(error);
    }
};

const API = {
    listRootDirectories
}

export default API;