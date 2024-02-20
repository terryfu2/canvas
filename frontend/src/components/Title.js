import styled from "styled-components";
import React from "react";

const Logo = styled.div`
  position: fixed;
  top: 1em;
  left: 2em;
  opacity: 0.3;
  transition: opacity 0.5s cubic-bezier(0.25, 0.8, 0.25, 1);
  font-weight: bold;
  &:hover {
    opacity: 1;
  }
  & > h1 {
    font-size: 40px;
    margin: 0 0;
    & > a {
      text-decoration: none;
      position: relative;
      color: #7ec4cf;
    }
  }
  @media (max-width: 960px) {
    position: relative;
    display: grid;
    left: -6em;
    font-size: 10px;
  }
`;
export const Title = () => (
  <Logo>
    <h1>
      <a href="">canvas</a>
    </h1>
  </Logo>
);